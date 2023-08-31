# Option 2: Introducing Spend Policies to Emulate HLTC Behavior. 

Sia is actively developing a new feature called "Spend Policies". These policies are intended to replace the current "UnlockCondition" functionality within Sia's consensus model.

These policies are an ideal candidate for introducing HLTC-like behavior.

As mentioned in option 1, we need to introduce logic that can impose the following validation rules onto a UTXO:

1. Alice can spend if she includes the correct secret

2. Bob can spend if a timestamp has passed

There is a planned spend policy that allows for conditional logic via "thresholds". This threshold policy allows locking UTXOs behind "n of m" statements. ie, "if 3 of 5 of the following policies are fulfilled, the UTXO can be spent"

We propose introducing two additional policy types, PolicyTypeHashLock and PolicyTypeTimeLock.

### PolicyTypeHashLock must:

1. Validate a provided secret against the `hash_secret` within the policy. ie, `sha256(secret) == hash_secret`

2. Validate that this provided secret is 32 bytes long.

### PolicyTypeTimeLock must:


1. Validate that the `timestamp` within the policy has passed.


We propose using this threshold mechanism to introduce HLTC-like behavior in the following way:

```
{
    "PolicyTypeThreshold": {
        "N": 1,
        "Of": [
            "PolicyTypeThreshold": {
                "N": 2,
                "Of": {
                    "PolicyTypePublicKey": "<alice_pubkey>",
                    "PolicyTypeHashLock": "<secret_hash>"
                }
            },
            "PolicyTypeThreshold": {
                "N": 2,
                "Of": {
                    "PolicyTypePublicKey": "<bob_pubkey>",
                    "PolicyTypeTimeLock": "<timestamp>"
                }
            }
        ]
    }
}
```