# fn-2.4: DID namespace pointer management

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/storage/git.rs` (extend) + `crates/radicle/src/identity/keri.rs`

## What to Do

When a user with a KERI identity pushes to a project, their identity namespace
`refs/namespaces/did-keri-<prefix>/refs/rad/id` must be written into the project repo,
pointing to the KERI identity repo's RID.

### Writing the Pointer

```rust
// In storage/git.rs or identity/keri.rs
pub fn write_identity_namespace(
    repo: &git2::Repository,
    identity_ns: &IdentityNamespace,
    identity_rid: &RepoId,
) -> Result<(), git2::Error> {
    let pointer = IdentityPointer { rid: *identity_rid };
    let blob_oid = pointer.write_blob(repo)?;

    // Build tree with one entry: "id" → blob_oid
    let mut tb = repo.treebuilder(None)?;
    tb.insert("id", blob_oid, git2::FileMode::Blob.into())?;
    let tree_oid = tb.write()?;

    // Create commit on refs/namespaces/did-keri-<prefix>/refs/rad/id
    let ref_name = identity_ns.rad_id_ref();
    let sig = repo.signature()?;
    let parents = // empty for first commit, or existing tip
    repo.commit(Some(&ref_name), &sig, &sig, "identity namespace", &tree, &parents)?;
    Ok(())
}
```

### Reading the Pointer

```rust
pub fn read_identity_pointer(
    repo: &git2::Repository,
    identity_ns: &IdentityNamespace,
) -> Result<Option<RepoId>, git2::Error> {
    let ref_name = identity_ns.rad_id_ref();
    let Ok(reference) = repo.find_reference(&ref_name) else { return Ok(None) };
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;
    let blob_entry = tree.get_name("id").ok_or(...)? ;
    let blob = blob_entry.to_object(repo)?.peel_to_blob()?;
    let pointer = IdentityPointer::read_blob(repo, blob.id())?;
    Ok(Some(pointer.rid))
}
```

### When is this Called?

- **Write**: When `rad identity init-keri` is run or when a KERI-identity-enabled device
  pushes `rad/sigrefs` to a project for the first time.
- **Read**: During fetch (fn-4.2) to discover which KERI identity repo to fetch.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] `write_identity_namespace()` creates the correct ref with the correct blob
- [ ] `read_identity_pointer()` reads back the same RID
- [ ] Round-trip test in a temp git repo
- [ ] Returns `None` (not error) when the ref does not exist

## Done summary
- Added `write_identity_namespace()` and `read_identity_pointer()` functions in `identity/namespace.rs`
- `write_identity_namespace`: creates commit with "id" blob on `refs/namespaces/did-keri-<prefix>/refs/rad/id`
- `read_identity_pointer`: reads blob, parses `RepoId`, returns `None` when ref absent
- Round-trip test and missing-ref test both pass
- Re-exported from `radicle::identity`
- Why: Namespace pointers link project repos to KERI identity repos during push/fetch
- Verification: `cargo test -p radicle -- identity::namespace` (7 tests pass)
## Evidence
- Commits:
- Tests: cargo test -p radicle -- identity::namespace
- PRs:
