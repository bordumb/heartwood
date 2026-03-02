use std::path::PathBuf;

use auths_radicle::{
    DefaultBridge, EnforcementMode, RadicleAuthsBridge, VerifyRequest, VerifyResult,
};
/// Returns the current time as an auths-radicle `Timestamp`.
fn now() -> auths_radicle::bridge::Timestamp {
    chrono::Utc::now()
}

use crate::crypto::PublicKey;
use crate::identity::{Did, Doc, RepoId};
use crate::storage::auths_adapter::HeartwooodAuthsStorage;

/// The result of checking a device's authority to sign refs for a project.
#[derive(Debug, Clone)]
pub enum DeviceAuthority {
    /// Device's key is directly listed as a delegate in the project Doc.
    DirectDelegate { did: Did },
    /// Device key is attested under a KERI identity that is a delegate.
    AttestedDevice {
        device_key: PublicKey,
        identity_did: Did,
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

/// Production implementation of `DeviceAuthorityChecker`.
///
/// Fast path: checks if the signer is a direct delegate in the project Doc.
/// Slow path: uses `RadicleAuthsBridge` to verify attestation via auths policy engine.
pub struct CompositeAuthorityChecker {
    bridge: DefaultBridge<HeartwooodAuthsStorage>,
}

impl CompositeAuthorityChecker {
    /// Create a new checker backed by Heartwood storage at the given path.
    pub fn new(storage_path: impl Into<PathBuf>) -> Self {
        let auths_storage = HeartwooodAuthsStorage::new(storage_path);
        Self {
            bridge: DefaultBridge::with_storage(auths_storage),
        }
    }
}

impl DeviceAuthorityChecker for CompositeAuthorityChecker {
    fn check(
        &self,
        signer: &PublicKey,
        doc: &Doc,
        repo_id: &RepoId,
    ) -> Result<DeviceAuthority, AuthorityError> {
        // Fast path: direct delegate check (no bridge call needed).
        let signer_did = Did::Key(*signer);
        if doc.is_delegate(&signer_did) {
            return Ok(DeviceAuthority::DirectDelegate { did: signer_did });
        }

        // Slow path: attestation lookup via the auths bridge.
        let request = VerifyRequest {
            signer_key: signer,
            repo_id,
            now: now(),
            mode: EnforcementMode::Enforce,
            known_remote_tip: None,
            min_kel_seq: None,
            required_capability: None,
        };

        let result = self
            .bridge
            .verify_signer(&request)
            .map_err(|e| AuthorityError::Bridge(e.to_string()))?;

        match result {
            VerifyResult::Verified { .. } => {
                let device_did = self.bridge.device_did(signer);
                // Find the KERI identity that controls this device.
                let identity_did = self
                    .bridge
                    .find_identity_for_device(&device_did, repo_id)
                    .map_err(|e| AuthorityError::Bridge(e.to_string()))?
                    .ok_or(AuthorityError::NotAuthorized)?;

                // Verify the KERI identity is a delegate in the project doc.
                if !doc.is_delegate(&identity_did) {
                    return Err(AuthorityError::NotAuthorized);
                }

                Ok(DeviceAuthority::AttestedDevice {
                    device_key: *signer,
                    identity_did,
                })
            }
            VerifyResult::Rejected { reason } => Err(AuthorityError::Rejected(reason)),
            VerifyResult::Warn { .. } => {
                // In observe mode the bridge downgrades rejections to warnings.
                // Still allow, but find identity for the result.
                let device_did = self.bridge.device_did(signer);
                let identity_did = self
                    .bridge
                    .find_identity_for_device(&device_did, repo_id)
                    .map_err(|e| AuthorityError::Bridge(e.to_string()))?
                    .ok_or(AuthorityError::NotAuthorized)?;

                Ok(DeviceAuthority::AttestedDevice {
                    device_key: *signer,
                    identity_did,
                })
            }
            VerifyResult::Quarantine { reason, .. } => Err(AuthorityError::Bridge(format!(
                "identity repo needs fetching: {reason}"
            ))),
            _ => Err(AuthorityError::Bridge("unexpected verify result".into())),
        }
    }
}
