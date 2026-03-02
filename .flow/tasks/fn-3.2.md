# fn-3.2: Implement `CompositeAuthorityChecker` using `RadicleAuthsBridge`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/identity/device_authority.rs` (extend)

## What to Do
## Action: Constants Reconciliation
**STRICT**: Heartwood MUST NOT define its own KERI or attestation ref strings.
1. Add `auths-radicle` as a dependency.
2. DELETE all hardcoded path strings (like `refs/keri/kel`) in Heartwood.
3. Import and use the `auths_radicle::refs` constants for all Git operations.

## Action: Configuration Injection
Update `CompositeAuthorityChecker` to initialize the bridge using a custom `StorageLayoutConfig` provided by `auths-radicle`:
```rust
// Use the custom path builder from auths-radicle
let layout = auths_radicle::refs::Layout::radicle(); 
let bridge = DefaultBridge::new(auths_storage, policy).with_layout(layout);
```


Implement the `DeviceAuthorityChecker` port using `auths_radicle::DefaultBridge` as the
backend. This is the production implementation.

```rust
use auths_radicle::{DefaultBridge, RadicleAuthsBridge, VerifyResult};
use auths_policy::PolicyBuilder;
use crate::storage::auths_adapter::HeartwooodAuthsStorage;

pub struct CompositeAuthorityChecker<S> {
    bridge: DefaultBridge<HeartwooodAuthsStorage<S>>,
}

impl<S: Storage + Send + Sync + 'static> CompositeAuthorityChecker<S> {
    /// Create with default policy: not_revoked + not_expired + sign_commit capability.
    pub fn new(storage: S) -> Self {
        let policy = PolicyBuilder::new()
            .not_revoked()
            .not_expired()
            .require_capability("sign_commit")
            .build();
        let auths_storage = HeartwooodAuthsStorage::new(storage);
        Self {
            bridge: DefaultBridge::new(auths_storage, policy),
        }
    }
}

impl<S: Storage + Send + Sync + 'static> DeviceAuthorityChecker for CompositeAuthorityChecker<S> {
    fn check(
        &self,
        signer: &PublicKey,
        doc: &Doc,
        repo_id: &RepoId,
        now: DateTime<Utc>,
    ) -> Result<DeviceAuthority, AuthorityError> {
        // 1. Fast path: direct delegate check
        let signer_did = Did::Key(*signer);
        if doc.is_delegate(&signer_did) {
            return Ok(DeviceAuthority::DirectDelegate { did: signer_did });
        }

        // 2. Slow path: attestation lookup via bridge
        let key_bytes: [u8; 32] = signer.as_bytes().try_into()
            .map_err(|_| AuthorityError::Bridge("invalid key length".to_string()))?;

        let result = self.bridge
            .verify_signer(&key_bytes, &repo_id.to_string(), now)
            .map_err(|e| AuthorityError::Bridge(e.to_string()))?;

        match result {
            VerifyResult::Verified { .. } => {
                // Find which KERI identity this device belongs to
                let identity_did = self.bridge
                    .device_did(&key_bytes);
                // Verify that KERI identity is a delegate in the doc
                if !doc.delegates.iter().any(|d| matches!(d, Did::Keri(k) if k == &identity_did)) {
                    return Err(AuthorityError::NotAuthorized);
                }
                Ok(DeviceAuthority::AttestedDevice {
                    device_key: *signer,
                    identity_did,
                    expires_at: None, // populated from attestation summary
                    capabilities: vec!["sign_commit".to_string()],
                })
            }
            VerifyResult::Rejected { reason } => Err(AuthorityError::Rejected(reason)),
            VerifyResult::Warn { reason } => {
                log::warn!("device authorization warning: {reason}");
                // Still allow, but return AttestedDevice
                Ok(DeviceAuthority::AttestedDevice { ... })
            }
        }
    }
}
```

## Key Invariant

**Radicle verifies crypto, Auths verifies authorization.** The call to
`self.bridge.verify_signer()` never re-verifies the Ed25519 signature — that was already
done by `SignedRefs::verify()` before this is called. The bridge only checks:
- Is there a valid attestation?
- Is it not revoked?
- Is it not expired?


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Direct delegate path returns `DirectDelegate` immediately (no bridge call)
- [ ] Attested device path returns `AttestedDevice` for valid attestations
- [ ] `Rejected` attestation returns `AuthorityError::Rejected`
- [ ] KERI identity that is NOT a project delegate returns `AuthorityError::NotAuthorized`
- [ ] Unit tests with mock bridge (test both paths)
