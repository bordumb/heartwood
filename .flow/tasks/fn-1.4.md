# fn-1.4: Add `IdentityPointer` blob type for `refs/rad/id`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/identity/namespace.rs` (extend) or new file `identity_pointer.rs`

## What to Do

The `refs/rad/id` ref inside an identity namespace contains a git blob whose content is
just the canonical RID of the KERI identity repo (e.g. `rad:z42hL2jL4XNk6K8oHQaSWfMgCL7ji`).

```rust
pub struct IdentityPointer {
    pub rid: RepoId,
}

impl IdentityPointer {
    /// Write the RID as a UTF-8 blob in `repo` and return its Oid.
    pub fn write_blob(&self, repo: &git2::Repository) -> Result<git2::Oid, git2::Error>

    /// Read and parse the blob at `oid` from `repo`.
    pub fn read_blob(repo: &git2::Repository, oid: git2::Oid) -> Result<Self, Error>
}
```

### Format

The blob content is the canonical string representation of the `RepoId` — the same format
that `RepoId::to_string()` produces (e.g. `rad:z42hL2jL4XNk6K8oHQaSWfMgCL7ji`). No
newline, no extra whitespace.

### Writing the ref

When a KERI identity namespace is created in a project repo:
1. Create the blob via `IdentityPointer::write_blob()`
2. Create a commit on `refs/namespaces/did-keri-<prefix>/refs/rad/id` whose tree
   contains this blob (no parent — initial commit)

This write logic lives in `GitKeriIdentityStore::init_identity_namespace()` (fn-2).


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [x] `IdentityPointer { rid }.write_blob()` creates a blob with the correct content
- [x] `IdentityPointer::read_blob(repo, oid)` parses back to the original `rid`
- [x] Round-trip test with a valid `RepoId`
- [x] Invalid blob content returns a parse error

## Done summary
- Extended `Did` from a newtype to an enum with `Key(PublicKey)` and `Keri(String)` variants
- Added `FromStr`/`Display` round-trip for both `did:key:` and `did:keri:` formats
- Added helper methods: `as_key()`, `as_keri_prefix()`, `is_rotatable()`, `to_ref_component()`
- Why: Foundation for multi-device identity support via KERI DIDs
- Verification: `cargo test -p radicle -- identity::did` passes (4 tests)
## Evidence
- Commits: 57f9c95d23337d677b79b7b230e14d9921c5b600
- Tests: cargo test -p radicle
- PRs: