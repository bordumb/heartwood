# fn-2.3: Implement `HeartwooodAuthsStorage` adapter

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## New File
`crates/radicle/src/storage/auths_adapter.rs`

## What to Do

Implement `auths_radicle::AuthsStorage` for Heartwood's storage layer. This is the
**adapter** that bridges the `auths-radicle` port to Heartwood's git repositories.

```rust
use auths_radicle::{AuthsStorage, BridgeError};
use auths_verifier::{Attestation, KeyState};

/// Adapter: implements auths-radicle's AuthsStorage port using Heartwood's git storage.
pub struct HeartwooodAuthsStorage<S> {
    storage: S,  // Heartwood's Storage<git2::Repository>
}

impl<S: Storage> HeartwooodAuthsStorage<S> {
    pub fn new(storage: S) -> Self
}

impl<S: Storage + Send + Sync> AuthsStorage for HeartwooodAuthsStorage<S> {

    fn load_key_state(&self, identity_did: &str) -> Result<KeyState, BridgeError> {
        // 1. Parse identity_did as a KeriDid to get the prefix
        // 2. Look up the KERI identity repo RID in local storage
        //    (search project repos for refs/namespaces/did-keri-<prefix>/refs/rad/id)
        // 3. Open GitKeriIdentityStore for that repo
        // 4. Return key_state()
    }

    fn load_attestation(&self, device_did: &str) -> Result<Attestation, BridgeError> {
        // 1. Convert device_did to NodeId (did:key:z6Mk... → PublicKey → NodeId)
        // 2. Find which identity repo contains refs/keys/<nid>/signatures
        // 3. Load and return attestation
    }

    fn find_identity_for_device(&self, device_did: &str) -> Result<String, BridgeError> {
        // 1. Convert device_did to NodeId
        // 2. Search identity repos for refs/keys/<nid>/signatures
        // 3. Return the KERI DID of the identity that attested this device
    }
}
```

### Key Behaviours

- `load_key_state`: The KERI repo RID is found by scanning known repos for
  `refs/namespaces/did-keri-<prefix>/refs/rad/id`. This scan should be fast (cached or
  indexed in a follow-up).
- `load_attestation`: Identity repos are the authoritative source. Project repos don't
  store attestations.
- `find_identity_for_device`: Scan all known identity repos. Return the one containing an
  attestation for this device.

### Error Mapping

`BridgeError` (from `auths-radicle`) must be returned. Map Heartwood-specific errors:
```rust
fn map_err(e: KeriStoreError) -> BridgeError {
    BridgeError::Storage(e.to_string())
}
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] `HeartwooodAuthsStorage::load_attestation()` returns correct data from git
- [ ] `HeartwooodAuthsStorage::find_identity_for_device()` locates the right identity repo
- [ ] Implements `AuthsStorage` bound (`Send + Sync`)
- [ ] Unit tests with mock storage (or test fixtures)
