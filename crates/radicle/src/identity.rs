#![warn(clippy::unwrap_used)]
pub mod crefs;
pub mod device_authority;
pub mod did;
pub mod doc;
pub mod keri;
pub mod namespace;
pub mod project;

pub use crefs::CanonicalRefs;
pub use crypto::PublicKey;
pub use device_authority::{AuthorityError, DeviceAuthority, DeviceAuthorityChecker};
pub use did::Did;
pub use doc::{Doc, DocAt, DocError, IdError, PayloadError, RawDoc, RepoId, Visibility};
pub use keri::{KeriIdentityStore, KeriStoreError};
pub use namespace::{
    discover_identity_refs, read_identity_pointer, write_identity_namespace, IdentityNamespace,
    IdentityPointer, IdentityPointerError, NamespaceKind,
};
pub use project::Project;

pub use crate::cob::identity::{Action, Error, Identity, IdentityMut, TYPENAME};
