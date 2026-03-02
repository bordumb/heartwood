# fn-2.2: Implement `GitKeriIdentityStore`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/identity/keri.rs` (extend file from fn-2.1)

## What to Do

Implement the `KeriIdentityStore` trait backed by a `git2::Repository`.

### Key ref paths (per RIP-X)

```rust
const KERI_KEL_REF: &str = "refs/keri/kel";
const KEYS_NAMESPACE: &str = "refs/keys";
// Per-device: refs/keys/<nid>/signatures
```

### Struct

```rust
pub struct GitKeriIdentityStore {
    repo: git2::Repository,
}

impl GitKeriIdentityStore {
    pub fn open(path: &Path) -> Result<Self, KeriStoreError>
    pub fn init(path: &Path) -> Result<Self, KeriStoreError>
}
```

### `key_state()`

Delegate to `auths_radicle`'s GitKel-compatible API.  The `auths-id` crate's `GitKel`
currently uses `refs/did/keri/<prefix>/kel` as its default ref path. Per fn-5 in the
`auths` repo (fn-A.5.2), it will be updated to support a custom ref path. Use that:

```rust
fn key_state(&self) -> Result<KeyState, KeriStoreError> {
    // Uses auths-id's GitKel pointed at refs/keri/kel
    let kel = auths_id::keri::GitKel::with_ref("refs/keri/kel", &self.repo);
    let events = kel.get_events()?;
    let state = auths_id::keri::replay_kel(&events)?;
    Ok(state)
}
```

### `load_attestation(nid)`

Read attestation blobs from `refs/keys/<nid>/signatures`:

```rust
fn load_attestation(&self, nid: &NodeId) -> Result<Attestation, KeriStoreError> {
    let ref_name = format!("refs/keys/{}/signatures", nid);
    let reference = self.repo.find_reference(&ref_name)?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;
    let did_key_blob = tree.get_name("did-key").ok_or(...)?.to_object(&self.repo)?.peel_to_blob()?;
    let did_keri_blob = tree.get_name("did-keri").ok_or(...)?.to_object(&self.repo)?.peel_to_blob()?;
    // Deserialize into Attestation using auths-verifier's format
    ...
}
```

### `store_attestation(nid, att)`

Write the two signature blobs and create a commit on `refs/keys/<nid>/signatures`:

```rust
fn store_attestation(&self, nid: &NodeId, att: &Attestation) -> Result<(), KeriStoreError> {
    let did_key_bytes = &att.device_signature;
    let did_keri_bytes = &att.identity_signature;
    // Create blobs, build tree, create commit on refs/keys/<nid>/signatures
    ...
}
```

### `list_devices()`

Enumerate `refs/keys/` to find all device refs:

```rust
fn list_devices(&self) -> Result<Vec<NodeId>, KeriStoreError> {
    self.repo.references_glob("refs/keys/*/signatures")?
        .map(|r| parse_nid_from_ref(r?.name().unwrap_or("")))
        .collect()
}
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] `GitKeriIdentityStore::init()` creates a valid bare git repo with correct structure
- [ ] `store_attestation()` + `load_attestation()` round-trip correctly
- [ ] `list_devices()` returns all stored NIDs
- [ ] `key_state()` replays KEL events correctly
- [ ] Integration test: init → store attestation → list → load
