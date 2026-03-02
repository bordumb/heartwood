use std::path::{Path, PathBuf};
use std::str::FromStr;

use auths_id::keri::KeyState;
use auths_radicle::storage::GitRadicleStorage;
use auths_radicle::refs::Layout;
use auths_radicle::{AuthsStorage, BridgeError};
use auths_verifier::core::Attestation;

use crate::git::raw;
use crate::identity::did::Did;
use crate::identity::doc::RepoId;
use crate::identity::namespace::IdentityNamespace;

/// Adapter: implements auths-radicle's `AuthsStorage` port using Heartwood's
/// git storage directory layout.
///
/// Scans Heartwood's storage directory to find KERI identity repos by DID
/// namespace refs, then delegates to `GitRadicleStorage` for the actual
/// identity and attestation operations.
pub struct HeartwooodAuthsStorage {
    storage_path: PathBuf,
    layout: Layout,
}

impl HeartwooodAuthsStorage {
    pub fn new(storage_path: impl Into<PathBuf>) -> Self {
        Self {
            storage_path: storage_path.into(),
            layout: Layout::radicle(),
        }
    }

    /// Parse a KERI DID prefix from a full DID string like `"did:keri:EXq5..."`.
    fn parse_keri_prefix(identity_did: &Did) -> Result<&str, BridgeError> {
        identity_did
            .as_keri_prefix()
            .ok_or_else(|| BridgeError::IdentityLoad(format!("not a KERI DID: {identity_did}")))
    }

    /// Find the identity repo path for a KERI DID by scanning project repos
    /// for namespace pointer refs (`refs/namespaces/did-keri-<prefix>/refs/rad/id`).
    fn find_identity_repo_path(&self, keri_prefix: &str) -> Result<PathBuf, BridgeError> {
        let ns = IdentityNamespace::new(Did::Keri(keri_prefix.to_string()));
        let rad_id_ref = ns.rad_id_ref();

        let entries = self.list_repo_dirs()?;
        for path in &entries {
            if let Some(rid) = self.read_namespace_pointer(path, &rad_id_ref)? {
                return Ok(self.storage_path.join(rid.canonical()));
            }
        }

        // Fallback: scan for repos that have a KEL ref directly (identity repos).
        for path in &entries {
            if self.has_kel_ref(path) {
                return Ok(path.clone());
            }
        }

        Err(BridgeError::IdentityLoad(format!(
            "no identity repo found for did:keri:{keri_prefix}"
        )))
    }

    /// Open the identity repo for a KERI DID as a `GitRadicleStorage`.
    fn open_identity_storage(
        &self,
        identity_did: &Did,
    ) -> Result<GitRadicleStorage, BridgeError> {
        let prefix = Self::parse_keri_prefix(identity_did)?;
        let repo_path = self.find_identity_repo_path(prefix)?;
        GitRadicleStorage::open(&repo_path, self.layout.clone())
    }

    /// List all repo directories under the storage path.
    fn list_repo_dirs(&self) -> Result<Vec<PathBuf>, BridgeError> {
        let entries = std::fs::read_dir(&self.storage_path)
            .map_err(|e| BridgeError::Repository(format!("can't list storage: {e}")))?;

        let mut dirs = Vec::new();
        for entry in entries {
            let entry =
                entry.map_err(|e| BridgeError::Repository(format!("read_dir error: {e}")))?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            }
        }
        Ok(dirs)
    }

    /// Try to read a namespace pointer ref from a repo. Returns the target
    /// `RepoId` if the ref exists and points to a valid pointer blob.
    fn read_namespace_pointer(
        &self,
        repo_path: &Path,
        rad_id_ref: &str,
    ) -> Result<Option<RepoId>, BridgeError> {
        let repo = match raw::Repository::open_bare(repo_path) {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        let reference = match repo.find_reference(rad_id_ref) {
            Ok(r) => r,
            Err(e) if e.code() == raw::ErrorCode::NotFound => return Ok(None),
            Err(e) => {
                return Err(BridgeError::Repository(format!(
                    "ref lookup error in {}: {e}",
                    repo_path.display()
                )))
            }
        };

        // Peel to commit → tree → first blob
        if let Ok(commit) = reference.peel_to_commit() {
            if let Ok(tree) = commit.tree() {
                for tree_entry in tree.iter() {
                    if let Ok(blob) = repo.find_blob(tree_entry.id()) {
                        if let Ok(content) = std::str::from_utf8(blob.content()) {
                            if let Ok(rid) = RepoId::from_str(content.trim()) {
                                return Ok(Some(rid));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Check if a repo has a KERI KEL ref (i.e. is an identity repo).
    fn has_kel_ref(&self, repo_path: &Path) -> bool {
        let Ok(repo) = raw::Repository::open_bare(repo_path) else {
            return false;
        };
        let found = repo
            .find_reference(auths_radicle::refs::KERI_KEL_REF)
            .is_ok();
        found
    }

    /// Enumerate DID namespace refs in a project repo, returning
    /// `(keri_did, identity_repo_path)` pairs.
    fn enumerate_identity_namespaces(
        &self,
        project_path: &Path,
    ) -> Result<Vec<(Did, PathBuf)>, BridgeError> {
        let repo = raw::Repository::open_bare(project_path).map_err(|e| {
            BridgeError::Repository(format!(
                "failed to open project repo at {}: {e}",
                project_path.display()
            ))
        })?;

        let glob = "refs/namespaces/did-keri-*/refs/rad/id";
        let refs = repo
            .references_glob(glob)
            .map_err(|e| BridgeError::Repository(format!("ref glob error: {e}")))?;

        let mut result = Vec::new();
        for r in refs {
            let reference = match r {
                Ok(r) => r,
                Err(_) => continue,
            };
            let name = match reference.name() {
                Some(n) => n,
                None => continue,
            };

            // Parse: refs/namespaces/did-keri-<prefix>/refs/rad/id
            let parts: Vec<&str> = name.split('/').collect();
            if parts.len() < 3 {
                continue;
            }
            let ns = match IdentityNamespace::from_ref_component(parts[2]) {
                Some(ns) => ns,
                None => continue,
            };
            let keri_did = ns.did().clone();

            // Read the pointer blob
            if let Ok(commit) = reference.peel_to_commit() {
                if let Ok(tree) = commit.tree() {
                    for tree_entry in tree.iter() {
                        if let Ok(blob) = repo.find_blob(tree_entry.id()) {
                            if let Ok(content) = std::str::from_utf8(blob.content()) {
                                if let Ok(rid) = RepoId::from_str(content.trim()) {
                                    let identity_path = self.storage_path.join(rid.canonical());
                                    result.push((keri_did.clone(), identity_path));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}

impl AuthsStorage for HeartwooodAuthsStorage {
    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn load_key_state(&self, identity_did: &Did) -> Result<KeyState, BridgeError> {
        self.open_identity_storage(identity_did)?
            .load_key_state(identity_did)
    }

    fn load_attestation(
        &self,
        device_did: &Did,
        identity_did: &Did,
    ) -> Result<Attestation, BridgeError> {
        self.open_identity_storage(identity_did)?
            .load_attestation(device_did, identity_did)
    }

    fn find_identity_for_device(
        &self,
        device_did: &Did,
        repo_id: &RepoId,
    ) -> Result<Option<Did>, BridgeError> {
        let project_path = self.storage_path.join(repo_id.canonical());

        for (keri_did, identity_path) in self.enumerate_identity_namespaces(&project_path)? {
            let identity_storage = match GitRadicleStorage::open(&identity_path, self.layout.clone()) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if let Ok(Some(_)) =
                identity_storage.find_identity_for_device(device_did, repo_id)
            {
                return Ok(Some(keri_did));
            }
        }

        Ok(None)
    }

    fn local_identity_tip(&self, identity_did: &Did) -> Result<Option<[u8; 20]>, BridgeError> {
        self.open_identity_storage(identity_did)?
            .local_identity_tip(identity_did)
    }
}
