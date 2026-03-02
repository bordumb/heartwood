use crate::crypto::PublicKey;
use crate::git::raw;

use auths_radicle::RadAttestation;

/// Port: read/write access to a KERI identity repository.
///
/// This is the only interface the rest of Heartwood uses for KERI operations.
/// Implemented by `GitKeriIdentityStore` (fn-2.2).
pub trait KeriIdentityStore: Send + Sync {
    /// List all attested device NodeIds (from `refs/keys/`).
    fn list_devices(&self) -> Result<Vec<PublicKey>, KeriStoreError>;

    /// Load an attestation for a device.
    fn load_attestation(&self, nid: &PublicKey) -> Result<RadAttestation, KeriStoreError>;

    /// Write a new attestation for a device.
    fn store_attestation(
        &self,
        nid: &PublicKey,
        att: &RadAttestation,
    ) -> Result<(), KeriStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum KeriStoreError {
    #[error("git error: {0}")]
    Git(#[from] raw::Error),
    #[error("attestation not found for {0}")]
    AttestationNotFound(PublicKey),
    #[error("KEL error: {0}")]
    Kel(String),
    #[error("serialization error: {0}")]
    Serde(String),
}
