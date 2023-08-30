# Alice starts with BTC.

- S_A - Alice ed25519 private key which may be leaked to Bob
- S_B - Bob ed25519 private key which may be leaked to Alice

- A - Alice ed25519 private key - not ever leaked to Bob
- B - Bob ed25519 private key - not ever leaked to Alice

- tx_lock_SIA
```
inputs:
	#0 - #n : bob_trade_amount + txfee*2
outputs:
	#0 : 2of2( ed25519_public_key(S_A) , ed25519_public_key(S_B) ) for bob_trade_amount+txfee
	#1 - #n : optional change outputs
```

- tx_refund_SIA
```
inputs:
	#0 : tx_lock_SIA_vout_0 for bob_trade_amount+txfee with signature.timelock = timelock*2
outputs:
	#0 - #n : Bob can choose any output configuration 
```
- tx_redeem_SIA
```
inputs:
	#0 : tx_lock_SIA_vout_0 for bob_trade_amount 
outputs:
	#0 - #n : Alice can choose any output configuration 
```
NOTE: The output configuation of tx_redeem_SIA must be communicated to Bob prior to him generating his signature. Alternatively, Bob can provide a signature that does not sign the outputs of the transaction via SIA's `coveredFields` mechanism. This mechanism is similar to bitcoin's "sighash flags" mechanism.

- tx_lock_BTC
```
inputs:
	#0 - #n : alice_trade_amount + txfee*2
outputs:
	#0 : P2SH or P2WSH of bob_trade_amount+txfee
		OP_IF
			<timelock*1> OP_CHECKLOCKTIMEVERIFY OP_DROP <public_key(A)> OP_CHECKSIG
		OP_ELSE
			<public_key(A)> OP_CHECKSIGVERIFY <public_key(B)> OP_CHECKSIG
		OP_ENDIF
	#1 - #n : optional change outputs
```

- tx_redeem_BTC
```
inputs:
	#0 : tx_lock_BTC_vout_0 for bob_trade_amount+txfee with script:
		<bob signature> <alice signature> OP_FALSE
		OP_IF
			<timelock*1> OP_CHECKLOCKTIMEVERIFY OP_DROP <public_key(A)> OP_CHECKSIG 
		OP_ELSE 
			<public_key(A)> OP_CHECKSIGVERIFY <public_key(B)> OP_CHECKSIG
		OP_ENDIF 
		OP_HASH160 <scripthash> OP_EQUAL
outputs:
	#0 - #n : Bob can choose any output configuration by comminucating it to Alice prior to her signing with SIGHASH_ALL or she can provide a SIGHASH_NONE signature 
```

## Sequence of a successful swap where Alice starts with BTC
### Transaction flow: Bob locks SIA -> Alice locks BTC -> Bob spends BTC -> Alice spends SIA

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

3. Bob validates the anti-spam transaction. 

4. Bob creates the unsigned transactions, tx_lock_SIA and tx_refund_SIA. Bob sends both to Alice.

5. Alice validates both transactions. Alice signs her portion of the tx_refund_SIA transaction. Alice sends this signature to Bob.

6. Bob validates Alice's signature of tx_refund_SIA. Bob now has the guarantee that if he signs and broadcasts tx_lock_SIA, he can get a refund after timelock\*2 by signing and broadcasting tx_refund_SIA if Alice bails out. Bob broadcasts tx_lock_SIA.

7. Alice creates and broadcasts tx_lock_BTC.

8. Alice generates an adaptor siganture for tx_redeem_BTC_vin_0, adaptor_sig(A, S_A). This is an incomplete signature signing her portion of the script, `<public_key(A)> OP_CHECKSIGVERIFY`. Alice sends this siganture to Bob.

9. Bob can now complete Alice's signature by adapting it with S_B. Bob can now sign and broadcast the transaction, tx_redeem_BTC. By doing so, he reveals S_B to Alice.

10. Alice combines S_A with the now revealed S_B and creates and broadcasts tx_redeem_SIA. Alice can keep this UTXO to be spent at any time in the future because only she knows S_A and S_B.

## Sequence of an unsuccessful swap where Alice starts with BTC, Bob locks SIA, but Alice goes offline or otherwise refuses to lock her BTC
### Transaction flow: Bob locks SIA -> Bob broadcasts tx_SIA_refund after timelock\*2
Steps 0 - 6. identical to the successful path

7. Alice is offline or otherwise refusing to lock the BTC. Bob broadcasts tx_refund_SIA after timelock*2.


## Sequence of an unsuccessful swap where Alice starts with BTC, both parties lock funds, but Bob does not reveal S_B to Alice
### Transaction flow: Bob locks SIA -> Alice locks BTC -> Alice refunds BTC after timelock\*1 -> Bob broadcasts tx_SIA_refund after timelock\*2
Steps 0 - 8. identical to the successful path

8. Bob is offline or otherwise refusing to claim the BTC. Alice waits until locktime\*1 has past and spends the BTC. 

9. Bob broadcasts tx_refund_SIA after timelock\*2.
