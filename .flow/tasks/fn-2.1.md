# fn-2.1: Add `KeriIdentityStore` trait and `KeriStoreError`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## New File
`crates/radicle/src/identity/keri.rs`

## What to Do

Define the port (trait) that represents operations on a KERI identity's git-backed storage.
This is the **only** interface the rest of Heartwood uses — it never touches `auths-id` or
`git2` directly for KERI operations.

```rust
use auths_radicle::{KeyState, Attestation};

/// Port: read/write access to a KERI identity repository.
/// Implemented by `GitKeriIdentityStore` (in fn-2.2).
pub trait KeriIdentityStore: Send + Sync {
    /// Current key state (replays the KEL).
    fn key_state(&self) -> Result<KeyState, KeriStoreError>;

    /// Load an attestation for a device NodeId.
    fn load_attestation(&self, nid: &NodeId) -> Result<Attestation, KeriStoreError>;

    /// List all attested device NodeIds (active and revoked).
    fn list_devices(&self) -> Result<Vec<NodeId>, KeriStoreError>;

    /// Write a new attestation for a device.
    fn store_attestation(&self, nid: &NodeId, att: &Attestation) -> Result<(), KeriStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum KeriStoreError {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("attestation not found for {0}")]
    AttestationNotFound(NodeId),
    #[error("kel error: {0}")]
    Kel(String),
    #[error("serialization error: {0}")]
    Serde(String),
}
```

### Re-export from `identity/mod.rs`

```rust
pub use keri::{KeriIdentityStore, KeriStoreError};
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Trait compiles with correct bounds (`Send + Sync`)
- [ ] `KeriStoreError` implements `std::error::Error`
- [ ] Re-exported from `radicle::identity`
