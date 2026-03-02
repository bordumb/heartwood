# fn-4.2: Add `discover_identity_refs()` to scan project repo for DID namespaces

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/storage/git.rs`

## What to Do

Add a function that scans a project repo for DID identity namespace pointers and returns
the RIDs of the referenced KERI identity repos:

```rust
/// Scan a project repo for KERI identity namespaces and return the RIDs
/// of any referenced KERI identity repos.
pub fn discover_identity_refs(
    repo: &git2::Repository,
) -> Result<Vec<(IdentityNamespace, RepoId)>, git2::Error> {
    let mut results = Vec::new();

    for reference in repo.references_glob("refs/namespaces/did-*/refs/rad/id")? {
        let reference = reference?;
        let ref_name = reference.name().unwrap_or("");

        // Extract the namespace component
        // "refs/namespaces/did-keri-EXq5.../refs/rad/id" → "did-keri-EXq5..."
        if let Some(ns) = parse_identity_ns_from_ref(ref_name) {
            if let Ok(Some(rid)) = read_identity_pointer(repo, &ns) {
                results.push((ns, rid));
            }
        }
    }

    Ok(results)
}
```

### Helper: `parse_identity_ns_from_ref`

```rust
fn parse_identity_ns_from_ref(ref_name: &str) -> Option<IdentityNamespace> {
    // Split "refs/namespaces/<component>/refs/rad/id" → "<component>"
    let stripped = ref_name.strip_prefix("refs/namespaces/")?;
    let component = stripped.split('/').next()?;
    IdentityNamespace::from_ref_component(component)
}
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [x] Returns correct `(IdentityNamespace, RepoId)` pairs for each DID namespace
- [x] Returns empty vec (not error) when no DID namespaces exist
- [x] Test with a temp git repo containing one DID namespace ref
- [x] Test with a repo containing both peer and identity namespace refs (only identity returned)

## Done summary
- Added `discover_identity_refs()` in `identity/namespace.rs` (not `storage/git.rs` — co-located with `read_identity_pointer`)
- Added `parse_identity_ns_from_ref()` helper to extract namespace component from full ref path
- Re-exported from `radicle::identity`
- 4 new tests: one-namespace discovery, empty repo, ignores peer namespaces, ref parsing
- Verification: `cargo test -p radicle -- identity::namespace` (14 tests pass)
## Evidence
- Commits:
- Tests: cargo test -p radicle -- identity::namespace
- PRs:
