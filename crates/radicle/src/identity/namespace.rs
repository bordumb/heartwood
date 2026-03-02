use std::str::FromStr;

use crate::git::raw;
use crate::identity::did::Did;
use crate::identity::doc::RepoId;

/// A git ref namespace component derived from a DID.
///
/// Per RIP-X:
/// - `did:keri:<prefix>` → `did-keri-<prefix>`
/// - `did:key:<pubkey>`  → `did-key-<pubkey>` (for consistency)
///
/// A ref component starting with `did-` indicates an identity namespace
/// rather than an ordinary peer namespace.
pub struct IdentityNamespace {
    did: Did,
}

impl IdentityNamespace {
    /// Wrap a DID into an `IdentityNamespace`.
    pub fn new(did: Did) -> Self {
        Self { did }
    }

    /// Return the git ref component, e.g. `"did-keri-EXq5..."` or `"did-key-z6Mk..."`.
    pub fn ref_component(&self) -> String {
        match &self.did {
            Did::Key(pk) => format!("did-key-{}", pk.to_human()),
            Did::Keri(prefix) => format!("did-keri-{}", prefix),
        }
    }

    /// Return the full namespace prefix, e.g. `"refs/namespaces/did-keri-EXq5..."`.
    pub fn ref_prefix(&self) -> String {
        format!("refs/namespaces/{}", self.ref_component())
    }

    /// Return the full refname for the `rad/id` pointer inside this identity namespace,
    /// e.g. `"refs/namespaces/did-keri-EXq5.../refs/rad/id"`.
    pub fn rad_id_ref(&self) -> String {
        format!("{}/refs/rad/id", self.ref_prefix())
    }

    /// Parse a ref component back into an `IdentityNamespace`, returning `None` for
    /// ordinary peer NIDs that do not start with `"did-"`.
    pub fn from_ref_component(component: &str) -> Option<Self> {
        if let Some(rest) = component.strip_prefix("did-keri-") {
            if rest.is_empty() {
                return None;
            }
            return Some(Self {
                did: Did::Keri(rest.to_owned()),
            });
        }
        if let Some(rest) = component.strip_prefix("did-key-") {
            let pk = crate::crypto::PublicKey::from_str(rest).ok()?;
            return Some(Self {
                did: Did::Key(pk),
            });
        }
        None
    }

    /// Return the underlying DID.
    pub fn did(&self) -> &Did {
        &self.did
    }
}

/// The content of the `refs/rad/id` blob inside a KERI identity namespace.
///
/// The blob is the canonical string representation of the `RepoId` of the
/// KERI identity repository (e.g. `rad:z42hL2jL4XNk6K8oHQaSWfMgCL7ji`).
pub struct IdentityPointer {
    pub rid: RepoId,
}

/// Error reading or parsing an [`IdentityPointer`] blob.
#[derive(Debug, thiserror::Error)]
pub enum IdentityPointerError {
    #[error("git error: {0}")]
    Git(#[from] raw::Error),
    #[error("blob is not valid UTF-8")]
    Utf8,
    #[error("invalid RepoId in blob: {0}")]
    RepoId(String),
}

impl IdentityPointer {
    /// Write the RID as a UTF-8 blob in `repo` and return its `Oid`.
    pub fn write_blob(&self, repo: &raw::Repository) -> Result<raw::Oid, raw::Error> {
        let content = self.rid.to_string();
        repo.blob(content.as_bytes())
    }

    /// Read and parse the blob at `oid` from `repo`.
    pub fn read_blob(
        repo: &raw::Repository,
        oid: raw::Oid,
    ) -> Result<Self, IdentityPointerError> {
        let blob = repo.find_blob(oid).map_err(IdentityPointerError::Git)?;
        let content = std::str::from_utf8(blob.content()).map_err(|_| IdentityPointerError::Utf8)?;
        let rid = RepoId::from_str(content)
            .map_err(|_| IdentityPointerError::RepoId(content.to_owned()))?;
        Ok(Self { rid })
    }
}

/// Write a DID namespace pointer into a project repo.
///
/// Creates (or updates) the ref `refs/namespaces/did-keri-<prefix>/refs/rad/id`
/// with a commit containing a blob that holds the identity repo's RID.
pub fn write_identity_namespace(
    repo: &raw::Repository,
    identity_ns: &IdentityNamespace,
    identity_rid: &RepoId,
) -> Result<(), IdentityPointerError> {
    let pointer = IdentityPointer { rid: *identity_rid };
    let blob_oid = pointer.write_blob(repo)?;

    let mut tb = repo.treebuilder(None).map_err(IdentityPointerError::Git)?;
    tb.insert("id", blob_oid, 0o100644)
        .map_err(IdentityPointerError::Git)?;
    let tree_oid = tb.write().map_err(IdentityPointerError::Git)?;
    drop(tb);

    let tree = repo
        .find_tree(tree_oid)
        .map_err(IdentityPointerError::Git)?;
    let sig = raw::Signature::now("radicle", "radicle@localhost")
        .map_err(IdentityPointerError::Git)?;

    let ref_name = identity_ns.rad_id_ref();
    let parent = repo
        .find_reference(&ref_name)
        .ok()
        .and_then(|r| r.peel_to_commit().ok());
    let parents: Vec<&raw::Commit<'_>> = parent.iter().collect();

    let commit_oid = repo
        .commit(None, &sig, &sig, "identity namespace", &tree, &parents)
        .map_err(IdentityPointerError::Git)?;
    repo.reference(&ref_name, commit_oid, true, "write identity namespace")
        .map_err(IdentityPointerError::Git)?;

    Ok(())
}

/// Read a DID namespace pointer from a project repo.
///
/// Returns the `RepoId` of the KERI identity repo that is bound to this
/// namespace, or `None` if the ref does not exist.
pub fn read_identity_pointer(
    repo: &raw::Repository,
    identity_ns: &IdentityNamespace,
) -> Result<Option<RepoId>, IdentityPointerError> {
    let ref_name = identity_ns.rad_id_ref();
    let reference = match repo.find_reference(&ref_name) {
        Ok(r) => r,
        Err(e) if e.code() == raw::ErrorCode::NotFound => return Ok(None),
        Err(e) => return Err(IdentityPointerError::Git(e)),
    };

    let commit = reference
        .peel_to_commit()
        .map_err(IdentityPointerError::Git)?;
    let tree = commit.tree().map_err(IdentityPointerError::Git)?;

    let entry = tree
        .get_name("id")
        .ok_or_else(|| IdentityPointerError::RepoId("missing 'id' entry in tree".into()))?;
    let pointer = IdentityPointer::read_blob(repo, entry.id())?;
    Ok(Some(pointer.rid))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_keri_ref_component_roundtrip() {
        let component = "did-keri-EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148";
        let ns = IdentityNamespace::from_ref_component(component).unwrap();
        assert_eq!(ns.ref_component(), component);
    }

    #[test]
    fn test_ordinary_nid_returns_none() {
        // An ordinary peer NID (z6Mk...) should not parse as an identity namespace.
        let nid = "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        assert!(IdentityNamespace::from_ref_component(nid).is_none());
    }

    #[test]
    fn test_rad_id_ref() {
        let component = "did-keri-EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148";
        let ns = IdentityNamespace::from_ref_component(component).unwrap();
        assert_eq!(
            ns.rad_id_ref(),
            "refs/namespaces/did-keri-EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148/refs/rad/id"
        );
    }

    #[test]
    fn test_key_ref_component_roundtrip() {
        let component = "did-key-z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let ns = IdentityNamespace::from_ref_component(component).unwrap();
        assert_eq!(ns.ref_component(), component);
    }

    #[test]
    fn test_empty_keri_prefix_returns_none() {
        assert!(IdentityNamespace::from_ref_component("did-keri-").is_none());
    }

    #[test]
    fn test_write_read_identity_namespace_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let repo = crate::git::raw::Repository::init_bare(dir.path()).unwrap();

        let ns = IdentityNamespace::new(Did::Keri("EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148".into()));
        let rid: RepoId = "rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5".parse().unwrap();

        write_identity_namespace(&repo, &ns, &rid).unwrap();
        let read_back = read_identity_pointer(&repo, &ns).unwrap();
        assert_eq!(read_back, Some(rid));
    }

    #[test]
    fn test_read_identity_pointer_returns_none_for_missing_ref() {
        let dir = tempfile::tempdir().unwrap();
        let repo = crate::git::raw::Repository::init_bare(dir.path()).unwrap();

        let ns = IdentityNamespace::new(Did::Keri("EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148".into()));
        let result = read_identity_pointer(&repo, &ns).unwrap();
        assert_eq!(result, None);
    }
}
