# Option 1: Introducing New Signature Schemes to Emulate HLTC Behavior. 

Sia supports adding additional signature schemes via soft forks. ref[https://github.com/SiaFoundation/core/blob/ad76cac3058febc60d5a0f2dfe000eb03a1977ca/consensus/validation.go#L523]

We believe it possible to emulate HLTC-like behavior with additional signature schemes being introduced into Sia's consensus model.

We need to introduce logic that can impose the following validation rules onto a UTXO:

1. Alice can spend if she includes the correct secret

2. Bob can spend if a timestamp has passed


With these rules in mind, we propose two additional signature schemes to be introduced, one for the "success" path of the HLTC and another for the "refund" path of the HLTC.

For demonstration purposes, a normal Sia address is generally a hashed representation of the following:
```
{
    "unlockConditions": {
        "timelock": 0,
        "publicKeys": [
            "ed25519:2fac394bd26ac986d05b54b8ab052fca209376e1955d2e19ae8297bde1d0e83b"
        ],
        "signaturesRequired": 1
    }
}

```

We propose adding functionality to support the following type of address:

```
{
    "unlockConditions": {
        "timelock": 0,
        "publicKeys": [
            "success:<alice_pubkey><hashed_secret>",
            "refund:<bob_pubkey><timestamp>"
        ],
        "signaturesRequired": 1
    }
}
```

Either party can spend this UTXO if they provide a signature and fulfill their condition, either providing a secret or waiting until a timestamp passes.

## Success path

The "signature" provided must be the ed25519 signature for `alice_pubkey` with a 32 byte `secret` appended to it. 

The validation logic for the the "success" signature scheme must:

1. Validate a provided signature against `alice_pubkey`

2. Validate a provided secret against the `hash_secret` within the unlockCondition. ie, `sha256(secret) == hash_secret`

3. Validate that this provided secret is 32 bytes long.

## Refund path

The "signature" provided must be simply the ed25519 signature for `bob_pubkey`

The validation logic for the the "success" signature scheme must:

1. Validate a provided signature against `bob_pubkey`

2. Validate that `timestamp` has passed.
