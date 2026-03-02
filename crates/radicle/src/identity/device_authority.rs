use crate::crypto::PublicKey;
use crate::identity::{Did, Doc, RepoId};

/// The result of checking a device's authority to sign refs for a project.
#[derive(Debug, Clone)]
pub enum DeviceAuthority {
    /// Device's key is directly listed as a delegate in the project Doc.
    DirectDelegate { did: Did },
    /// Device key is attested under a KERI identity that is a delegate.
    AttestedDevice {
        device_key: PublicKey,
        identity_did: String, // "did:keri:<prefix>"
    },
}

/// Error returned when a device authority check fails.
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
    ) -> Result<DeviceAuthority, AuthorityError>;
}
