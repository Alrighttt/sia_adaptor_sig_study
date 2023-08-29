use conquer_once::Lazy;
use curve25519_dalek::scalar::Scalar as Ed25519Scalar;
use rand::rngs::OsRng;
use rand_chacha::ChaCha20Rng;
use sha2::Sha256;
use sigma_fun::ext::dl_secp256k1_ed25519_eq::CrossCurveDLEQ;
use sigma_fun::HashTranscript;
use ecdsa_fun::adaptor::{Adaptor, EncryptedSignature};
use ecdsa_fun::fun::{Point as EcdsaPoint, Scalar as EcdsaScalar};
use ecdsa_fun::nonce::Deterministic;
use ecdsa_fun::{Signature as EcdsaSignature, ECDSA};

// https://github.com/comit-network/xmr-btc-swap/tree/20afb35d5b39d983cc08f82d54b5a76946526b0d
static CROSS_CURVE_PROOF_SYSTEM: Lazy<CrossCurveDLEQ<HashTranscript<Sha256, ChaCha20Rng>>> =
    Lazy::new(|| {
        CrossCurveDLEQ::<HashTranscript<Sha256, ChaCha20Rng>>::new(
            (*ecdsa_fun::fun::G).normalize(),
            curve25519_dalek::constants::ED25519_BASEPOINT_POINT,
        )
    });

struct Secp256k1KeyPair {
    private: EcdsaScalar,
    public: EcdsaPoint,
}

impl Secp256k1KeyPair {
    fn new_random(rng: &mut OsRng) -> Self {
        let scalar = EcdsaScalar::random(rng);

        let ecdsa = ECDSA::<()>::default();
        let public = ecdsa.verification_key_for(&scalar);

        Self {
            private: scalar,
            public,
        }
    }
    fn sign(&self, message_hash: &[u8; 32]) -> EcdsaSignature {
        let ecdsa = ECDSA::<Deterministic<Sha256>>::default();
        ecdsa.sign(&self.private, message_hash)
    }
}

fn ed25519_scalar_to_ecdsa_scalar(ed25519_scalar: &Ed25519Scalar) -> EcdsaScalar {
    let mut little_endian_bytes = ed25519_scalar.to_bytes();
    little_endian_bytes.reverse();
    let big_endian_bytes = little_endian_bytes;
    EcdsaScalar::from_bytes(big_endian_bytes)
        .unwrap()
        .non_zero()
        .unwrap()
}

fn main() {
    let mut rng = OsRng;
    let ecdsa = ECDSA::verify_only();

    // generate a one-time-use secp256k1 private key for each party
    // these keys are never leaked to the other party
    let a = Secp256k1KeyPair::new_random(&mut rng);
    let b = Secp256k1KeyPair::new_random(&mut rng);

    // generate a one-time-use ed25519 private key for each party
    // "secret_alice" "secret_bob"
    // these keys can be leaked to the other party via an adaptor signature
    let s_a: Ed25519Scalar = Ed25519Scalar::random(&mut rng);
    let s_b: Ed25519Scalar = Ed25519Scalar::random(&mut rng);

    // create a proof that this private key is valid on both ed25519 and secp256k1 curves
    // this will panic if the scalar is not on either curve
    let (s_a_dleq_proof, (s_a_public_secp256k1, s_a_public_ed25519)) =
        CROSS_CURVE_PROOF_SYSTEM.prove(&s_a, &mut rng);
    let (s_b_dleq_proof, (s_b_public_secp256k1, s_b_public_ed25519)) =
        CROSS_CURVE_PROOF_SYSTEM.prove(&s_b, &mut rng);

    // both parties exchange x.public, s_x_dleq_proof, s_x_public_secp256k1, s_x_public_ed25519
    // to each other
    let validate_alice_proof = CROSS_CURVE_PROOF_SYSTEM
        .verify(&s_a_dleq_proof, (s_a_public_secp256k1, s_a_public_ed25519));
    let validate_bob_proof = CROSS_CURVE_PROOF_SYSTEM
        .verify(&s_b_dleq_proof, (s_b_public_secp256k1, s_b_public_ed25519));
    // both parties validate the other's proof
    assert!(validate_alice_proof);
    assert!(validate_bob_proof);

    // one party generates an adaptor signature that will compel the other to reveal s_a or s_b
    let adaptor =
        Adaptor::<HashTranscript<Sha256, rand_chacha::ChaCha20Rng>, Deterministic<Sha256>>::default(
        );

    // FIRST CASE: Alice starts with BTC-like coin
    // Alice creates an adaptor signature and a normal signature signing `sighash`
    // This adaptor signature is not a valid signature until Bob adapts it with s_b.
    let example_sighash = [0x0; 32];
    let alice_adaptor_signature: EncryptedSignature =
        adaptor.encrypted_sign(&a.private, &s_b_public_secp256k1, &example_sighash);

    // Bob verifies alice_adaptor_signature will be a valid signature if adapted with s_b
    assert!(adaptor.verify_encrypted_signature(
        &a.public,
        &s_b_public_secp256k1,
        &example_sighash,
        &alice_adaptor_signature,
    ));

    // Bob adapts alice_adaptor_signature with s_b
    // Bob broadcasts this and `alice_signature` to spend the BTC UTXO
    let decrypted_signature = adaptor.decrypt_signature(
        &ed25519_scalar_to_ecdsa_scalar(&s_b),
        alice_adaptor_signature.clone(),
    );

    // Bob broadcasts this signature and a signature of his own to spend the BTC UTXO
    assert!(ecdsa.verify(&a.public, &example_sighash, &decrypted_signature));

    // Alice extracts s_b allowing her to spend the SIA UTXO
    let extracted_s_b = adaptor
        .recover_decryption_key(
            &s_b_public_secp256k1,
            &decrypted_signature,
            &alice_adaptor_signature,
        )
        .expect("signature must decrypt");
    assert_eq!(extracted_s_b, ed25519_scalar_to_ecdsa_scalar(&s_b));


    // OTHER CASE: Alices starts with SIA
    // Bob creates an adaptor signature and a normal signature signing `sighash`
    // This adaptor signature is not a valid signature until Alice adapts it with s_a.
    let example_sighash = [0x1; 32];
    let bob_adaptor_signature: EncryptedSignature =
        adaptor.encrypted_sign(&b.private, &s_a_public_secp256k1, &example_sighash);

    // Alice verifies bob_adaptor_signature will be a valid signature if adapted with s_a
    assert!(adaptor.verify_encrypted_signature(
        &b.public,
        &s_a_public_secp256k1,
        &example_sighash,
        &bob_adaptor_signature,
    ));

    // Alice adapts bob_adaptor_signature with s_a
    let decrypted_signature = adaptor.decrypt_signature(
        &ed25519_scalar_to_ecdsa_scalar(&s_a),
        bob_adaptor_signature.clone(),
    );

    // Alice broadcasts this signature and a signature of her own to spend the BTC UTXO
    assert!(ecdsa.verify(&b.public, &example_sighash, &decrypted_signature));

    // Bob extracts s_a allowing him to spend the SIA UTXO
    let extracted_s_a = adaptor
        .recover_decryption_key(
            &s_a_public_secp256k1,
            &decrypted_signature,
            &bob_adaptor_signature,
        )
        .expect("signature must decrypt");
    assert_eq!(extracted_s_a, ed25519_scalar_to_ecdsa_scalar(&s_a));
}
