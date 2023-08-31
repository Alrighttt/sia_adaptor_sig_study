# Alice starts with SIA.

- S_A - Alice ed25519 private key which may be leaked to Bob
- S_B - Bob ed25519 private key which may be leaked to Alice

- A - Alice secp256k1 private key - not ever leaked to Bob
- B - Bob secp256k1 private key - not ever leaked to Alice

- tx_lock_SIA
```
inputs:
	#0 - #n : alice_trade_amount + txfee*2
outputs:
	#0 : 2of2( ed25519_public_key(S_A) , ed25519_public_key(S_B) ) for alice_trade_amount+txfee
	#1 - #n : optional change outputs
```

- tx_refund_SIA
```
inputs:
	#0 : tx_lock_SIA_vout_0 for alice_trade_amount+txfee with signature.timelock = timelock*2
outputs:
	#0 - #n : Alice can choose any output configuration 
```

- tx_redeem_SIA
```
inputs:
	#0 : tx_lock_SIA_vout_0 for alice_trade_amount
outputs:
	#0 - #n : Bob can choose any output configuration 
```
NOTE: The output configuation of tx_redeem_SIA must be communicated to Alice prior to her generating her signature. Alternatively, Alice can provide a signature that does not sign the outputs of the transaction via SIA's `coveredFields` mechanism. This mechanism is similar to bitcoin's "sighash flags" mechanism.

- tx_lock_BTC
```
inputs:
	#0 - #n : bob_trade_amount + txfee*2
outputs:
	#0 : P2SH or P2WSH of bob_trade_amount+txfee
		OP_IF
			<timelock*1> OP_CHECKLOCKTIMEVERIFY OP_DROP <public_key(B)> OP_CHECKSIG
		OP_ELSE
			<public_key(A)> OP_CHECKSIGVERIFY <public_key(B)> OP_CHECKSIG
		OP_ENDIF
	#1 - #n : optional change outputs
```

- tx_redeem_BTC
```
inputs:
	#0 : tx_lock_BTC_vout_0 for bob_trade_amount+txfee with script:
		<bob signature> <alice signature> OP_FALSE OP_PUSHDATA1 OP_TOALTSTACK
		OP_IF
			<timelock*1> OP_CHECKLOCKTIMEVERIFY OP_DROP <public_key(B)> OP_CHECKSIG
		OP_ELSE 
			<public_key(A)> OP_CHECKSIGVERIFY <public_key(B)> OP_CHECKSIG
		OP_ENDIF 
		OP_HASH160 <scripthash> OP_EQUAL
outputs:
	#0 - #n : Alice can choose any output configuration by comminucating it to Bob prior to him signing with SIGHASH_ALL or he can provide a SIGHASH_NONE signature 
```

## Sequence of a succesful swap where Alice starts with SIA
Transaction flow: Bob locks BTC -> Alice locks SIA -> Alice spends BTC -> Bob spends SIA

0. Order matching. Both parties agree on volume, price and timelock duration.

1. Both parties generate, exchange and validate keys and proofs:

Alice sends:
```
ecdsa_public_key(S_A) - the ecdsa public key from private key, S_A
ed25519_public_key(S_A) - the ed25519 public key from private key, S_A
DLEQ_proof - a zero knowledge proof that ecdsa_public_key(S_A) and ed25519_public_key(S_A) are both generated from the same private key, S_A
public_key(A) - the public key from private key, A
```

Bob sends:
```
ecdsa_public_key(S_B) - the ecdsa public key from private key, S_B
ed25519_public_key(S_B) - the ed25519 public key from private key, S_B
DLEQ_proof - a zero knowledge proof that ecdsa_public_key(S_B) and ed25519_public_key(S_B) are both generated from the same private key, S_B
public_key(B) - the public key from private key, B
```


2. DEX fee or anti-spam fee is paid by Alice. Bob will initiate the swap by locking his coins, so Alice must suffer if she decides to spam Bob with malicious trade requests.

3. Bob validates the anti-spam transaction and signs and broadcasts tx_lock_BTC.

4. Alice validates, waits for confirmations and creates the unsigned transactions, tx_lock_SIA and tx_refund_SIA. Alice sends both unsigned transactions to Bob.

5. Bob validates both transactions. Bob signs his portion of the tx_refund_SIA transaction. Bob sends this signature to Alice.

6. Alice validates Bob's signature of tx_refund_SIA. Alice now has the guarantee that if she signs and broadcasts tx_lock_SIA, she can get a refund after timelock\*2 by signing and broadcasting tx_refund_SIA if bob bails out. Alice broadcasts tx_lock_SIA.

7. Bob generates an adaptor signature for tx_redeem_BTC_vin_0, adaptor_sig(B, S_A). This is an incomplete signature signing his portion of the script, `<public_key(B)> OP_CHECKSIG`. Bob sends this siganture to Alice.

8. Alice can now complete Bob's signature by adapting it with S_A. She now signs and broadcasts the complete transaction, tx_redeem_BTC. By doing so, she reveals S_A to Bob.

9. Bob combines S_B with the now revealed S_A and creates and broadcasts tx_redeem_SIA. Bob can keep this UTXO to be spent at any time in the future because only he knows S_A and S_B.

## Sequence of an unsuccesful swap where Alice starts with SIA, Bob locks BTC, but Alice goes offline or otherwise refuses to lock her SIA
Transaction flow: Bob locks BTC -> Bob refunds BTC after timelock\*1

Steps 0 - 5. identical to the successful path

8. Alice is offline or otherwise refusing to lock the SIA. Bob waits until locktime\*1 has past and spends the BTC. 

9. Alice broadcasts tx_refund_SIA after timelock\*2. 

## Sequence of a unsuccesful swap where Alice starts with SIA, both parties lock funds, but Alice does not reveal S_A to Bob
Transaction flow: Bob locks BTC -> Alice locks SIA -> Bob refunds BTC after timelock\*1 -> Alice broadcasts tx_SIA_refund after timelock\*2

Steps 0 - 7. identical to the successful path

8. Alice is offline or otherwise refusing to continue.

9. Bob waits until locktime\*1 has past and spends the BTC.

10. Alice broadcasts tx_refund_SIA after timelock\*2.
