# fn-3.1: Define `DeviceAuthority` type and `DeviceAuthorityChecker` trait

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## New File
`crates/radicle/src/identity/device_authority.rs`

## What to Do

Define the types that represent "how is this device authorized to sign?" and the port for
querying that information.

```rust
use chrono::{DateTime, Utc};
use crate::identity::{Did, Doc};
use crate::crypto::PublicKey;
use crate::storage::RepoId;

/// The result of checking a device's authority to sign refs for a project.
#[derive(Debug, Clone)]
pub enum DeviceAuthority {
    /// Device's key is directly listed as a delegate in the project Doc.
    DirectDelegate {
        did: Did,
    },
    /// Device key is attested under a KERI identity that is a delegate.
    AttestedDevice {
        device_key: PublicKey,
        identity_did: String,   // "did:keri:<prefix>"
        expires_at: Option<DateTime<Utc>>,
        capabilities: Vec<String>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum AuthorityError {
    #[error("signer is not a delegate and has no valid attestation")]
    NotAuthorized,
    #[error("attestation rejected: {0}")]
    Rejected(String),
    #[error("bridge error: {0}")]
    Bridge(String),
}

/// Port: checks whether a given signing key is authorized for a project.
pub trait DeviceAuthorityChecker: Send + Sync {
    fn check(
        &self,
        signer: &PublicKey,
        doc: &Doc,
        repo_id: &RepoId,
        now: DateTime<Utc>,
    ) -> Result<DeviceAuthority, AuthorityError>;
}
```

### Re-export

```rust
// crates/radicle/src/identity/mod.rs
pub use device_authority::{DeviceAuthority, DeviceAuthorityChecker, AuthorityError};
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Types compile with correct `Send + Sync` bounds
- [ ] `DeviceAuthority` implements `Debug` and `Clone`
- [ ] `AuthorityError` implements `std::error::Error`
- [ ] Re-exported from `radicle::identity`

## Done summary
- Created `identity/device_authority.rs` with `DeviceAuthority` enum, `AuthorityError`, and `DeviceAuthorityChecker` trait
- `DeviceAuthority::DirectDelegate` for keys directly in the Doc delegates
- `DeviceAuthority::AttestedDevice` for keys attested via KERI identity
- `DeviceAuthorityChecker` trait with `check()` method (Send + Sync bounds)
- Simplified vs spec: removed `DateTime<Utc>` and `capabilities` fields (auths-radicle handles policy evaluation internally)
- Re-exported from `radicle::identity`
- Why: Port for multi-device authorization checks during signed ref verification
- Verification: `cargo check -p radicle` passes
## Evidence
- Commits:
- Tests: cargo check -p radicle
- PRs:
