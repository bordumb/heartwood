use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use crate::crypto::PublicKey;
use crate::git::raw;

use auths_radicle::RadAttestation;

/// Port: read/write access to a KERI identity repository.
///
/// This is the only interface the rest of Heartwood uses for KERI operations.
/// Implemented by `GitKeriIdentityStore`.
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
    #[error("bridge error: {0}")]
    Bridge(String),
}

/// Git-backed implementation of `KeriIdentityStore`.
///
/// Wraps a bare git repository that follows the RIP-X identity layout:
/// - `refs/keri/kel` — KEL commit chain
/// - `refs/keys/<nid>/signatures/` — per-device attestation blobs
pub struct GitKeriIdentityStore {
    repo: Mutex<raw::Repository>,
}

impl GitKeriIdentityStore {
    /// Open an existing KERI identity repo at the given path.
    pub fn open(path: &Path) -> Result<Self, KeriStoreError> {
        let repo =
            raw::Repository::open_bare(path).map_err(|e| KeriStoreError::Bridge(e.to_string()))?;
        Ok(Self {
            repo: Mutex::new(repo),
        })
    }

    /// Initialize a new bare KERI identity repo at the given path.
    pub fn init(path: &Path) -> Result<Self, KeriStoreError> {
        let repo =
            raw::Repository::init_bare(path).map_err(|e| KeriStoreError::Bridge(e.to_string()))?;
        Ok(Self {
            repo: Mutex::new(repo),
        })
    }

    fn lock_repo(&self) -> std::sync::MutexGuard<'_, raw::Repository> {
        self.repo.lock().expect("repository mutex poisoned")
    }
}

impl KeriIdentityStore for GitKeriIdentityStore {
    fn list_devices(&self) -> Result<Vec<PublicKey>, KeriStoreError> {
        let repo = self.lock_repo();
        let glob = format!("{}/*/{}",
            auths_radicle::refs::KEYS_PREFIX,
            auths_radicle::refs::SIGNATURES_DIR,
        );
        let refs = repo
            .references_glob(&glob)
            .map_err(|e| KeriStoreError::Bridge(e.to_string()))?;

        let mut devices = Vec::new();
        for r in refs {
            let reference = r.map_err(|e| KeriStoreError::Bridge(e.to_string()))?;
            if let Some(name) = reference.name() {
                // Extract NID from refs/keys/<nid>/signatures
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() >= 3 {
                    let nid_str = parts[2]; // refs / keys / <nid> / signatures
                    if let Ok(pk) = PublicKey::from_str(nid_str) {
                        devices.push(pk);
                    }
                }
            }
        }
        Ok(devices)
    }

    fn load_attestation(&self, nid: &PublicKey) -> Result<RadAttestation, KeriStoreError> {
        let nid_str = nid.to_human();
        let repo = self.lock_repo();

        let dk_ref = auths_radicle::refs::device_did_key_ref(&nid_str);
        let dkeri_ref = auths_radicle::refs::device_did_keri_ref(&nid_str);

        let dk_blob = read_blob_content(&repo, &dk_ref)?
            .ok_or_else(|| KeriStoreError::AttestationNotFound(*nid))?;
        let dkeri_blob = read_blob_content(&repo, &dkeri_ref)?
            .ok_or_else(|| KeriStoreError::AttestationNotFound(*nid))?;

        let payload = auths_radicle::RadCanonicalPayload {
            did: String::new(), // Filled by caller with identity DID
            rid: String::new(), // Filled by caller with repo RID
        };

        let device_did = format!("did:key:{nid_str}");
        let device_pk_bytes: [u8; 32] = nid.as_ref().try_into().map_err(|_| {
            KeriStoreError::Serde("device key is not 32 bytes".into())
        })?;

        RadAttestation::from_blobs(&dk_blob, &dkeri_blob, payload, device_did, device_pk_bytes)
            .map_err(|e| KeriStoreError::Serde(e.to_string()))
    }

    fn store_attestation(
        &self,
        nid: &PublicKey,
        att: &RadAttestation,
    ) -> Result<(), KeriStoreError> {
        let nid_str = nid.to_human();
        let repo = self.lock_repo();
        let (dk_bytes, dkeri_bytes) = att.to_blobs();

        let dk_ref = auths_radicle::refs::device_did_key_ref(&nid_str);
        let dkeri_ref = auths_radicle::refs::device_did_keri_ref(&nid_str);

        write_blob_commit(&repo, &dk_ref, auths_radicle::refs::DID_KEY_BLOB, &dk_bytes)?;
        write_blob_commit(&repo, &dkeri_ref, auths_radicle::refs::DID_KERI_BLOB, &dkeri_bytes)?;

        Ok(())
    }
}

/// Read a blob's content from a ref that points to a commit with a single-entry tree.
fn read_blob_content(
    repo: &raw::Repository,
    ref_path: &str,
) -> Result<Option<Vec<u8>>, KeriStoreError> {
    let reference = match repo.find_reference(ref_path) {
        Ok(r) => r,
        Err(e) if e.code() == raw::ErrorCode::NotFound => return Ok(None),
        Err(e) => return Err(KeriStoreError::Bridge(format!("ref lookup error: {e}"))),
    };

    let commit = reference
        .peel_to_commit()
        .map_err(|e| KeriStoreError::Bridge(format!("ref not a commit: {e}")))?;
    let tree = commit
        .tree()
        .map_err(|e| KeriStoreError::Bridge(format!("missing tree: {e}")))?;

    let blob_name = ref_path.rsplit('/').next().unwrap_or(ref_path);
    let result = match tree.get_name(blob_name) {
        Some(entry) => {
            let blob = repo
                .find_blob(entry.id())
                .map_err(|e| KeriStoreError::Bridge(format!("blob read: {e}")))?;
            Ok(Some(blob.content().to_vec()))
        }
        None => Ok(None),
    };
    result
}

/// Write a blob into a new commit and point a ref at it.
fn write_blob_commit(
    repo: &raw::Repository,
    ref_path: &str,
    blob_name: &str,
    content: &[u8],
) -> Result<(), KeriStoreError> {
    let blob_oid = repo.blob(content)?;
    let mut tb = repo
        .treebuilder(None)
        .map_err(|e| KeriStoreError::Bridge(e.to_string()))?;
    tb.insert(blob_name, blob_oid, 0o100644)
        .map_err(|e| KeriStoreError::Bridge(e.to_string()))?;
    let tree_oid = tb.write().map_err(|e| KeriStoreError::Bridge(e.to_string()))?;
    drop(tb);

    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| KeriStoreError::Bridge(e.to_string()))?;
    let sig = raw::Signature::now("radicle", "radicle@localhost")
        .map_err(|e| KeriStoreError::Bridge(e.to_string()))?;

    let parent = repo.find_reference(ref_path).ok().and_then(|r| {
        r.peel_to_commit().ok()
    });
    let parents: Vec<&raw::Commit<'_>> = parent.iter().collect();

    let commit_oid = repo
        .commit(None, &sig, &sig, blob_name, &tree, &parents)
        .map_err(|e| KeriStoreError::Bridge(e.to_string()))?;

    repo.reference(ref_path, commit_oid, true, "store attestation")
        .map_err(|e| KeriStoreError::Bridge(e.to_string()))?;

    Ok(())
}
