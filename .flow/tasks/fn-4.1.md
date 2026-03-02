# fn-4.1: Add `NamespaceKind` enum + parsing in `storage/git.rs`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/storage/git.rs`

## What to Do

Add an enum to classify git namespace components so all namespace-type decisions are
centralized in one place (DRY principle):

```rust
/// Classification of a `refs/namespaces/<component>` entry.
pub enum NamespaceKind {
    /// A peer device namespace: refs/namespaces/<nid>/...
    /// The component is a valid NodeId (multibase public key).
    Peer(NodeId),
    /// A KERI identity namespace: refs/namespaces/did-keri-<prefix>/...
    /// Components starting with "did-" are identity namespaces.
    Identity(IdentityNamespace),
}

impl NamespaceKind {
    /// Parse a namespace component into its kind.
    /// Returns `None` if the component is neither a valid NodeId nor a recognized DID format.
    pub fn from_component(component: &str) -> Option<Self> {
        if let Ok(nid) = component.parse::<NodeId>() {
            return Some(NamespaceKind::Peer(nid));
        }
        if let Some(ns) = IdentityNamespace::from_ref_component(component) {
            return Some(NamespaceKind::Identity(ns));
        }
        None
    }

    pub fn as_peer(&self) -> Option<&NodeId> { ... }
    pub fn as_identity(&self) -> Option<&IdentityNamespace> { ... }
}
```

### Update Existing Namespace Scans

Find existing code that iterates `refs/namespaces/` and update to use `NamespaceKind`:

```rust
// Before:
for nid in storage.remotes()? { ... }

// After: callers can filter by kind
for ns_kind in storage.namespace_kinds()? {
    match ns_kind {
        NamespaceKind::Peer(nid) => { /* existing logic */ }
        NamespaceKind::Identity(ns) => { /* new: skip or handle */ }
    }
}
```

Make sure existing `Peer` filtering still works — no regressions in code that only cares
about peer namespaces.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] `NamespaceKind::from_component("z6MknSLr...")` returns `Peer` variant
- [ ] `NamespaceKind::from_component("did-keri-EXq5...")` returns `Identity` variant
- [ ] `NamespaceKind::from_component("unknown-garbage")` returns `None`
- [ ] Existing namespace iteration code still works (no regressions)
- [ ] Unit tests for all three cases
