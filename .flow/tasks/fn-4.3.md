# fn-4.3: Add `fetch_identity_repos()` to `radicle-fetch`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle-fetch/src/` (identify the right module from existing fetch code)

## What to Do

After a project repo is fetched, run identity repo discovery and fetch any referenced
KERI identity repos.

```rust
/// After fetching a project repo, discover and fetch any linked KERI identity repos.
/// This is best-effort: a failure here does not abort the project fetch result.
pub fn fetch_linked_identity_repos<S: Storage>(
    storage: &S,
    project_rid: &RepoId,
    peer: &NodeId,
    fetcher: &dyn Fetcher,
) -> Vec<Result<RepoId, FetchError>> {
    let repo = match storage.repository(*project_rid) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let identity_refs = match discover_identity_refs(repo.raw()) {
        Ok(refs) => refs,
        Err(_) => return vec![],
    };

    identity_refs.into_iter().map(|(_, rid)| {
        if storage.contains(rid) {
            // Already have it — just fetch updates
            fetcher.fetch(rid, peer)
        } else {
            // Clone the identity repo
            fetcher.clone(rid, peer)
        }
    }).collect()
}
```

### Integration Point

Find the main fetch entry point in `radicle-fetch` and add the identity repo fetch step
after a successful project repo fetch. Something like:

```rust
// After: let result = fetch_project(storage, rid, peer)?;
// Add:
fetch_linked_identity_repos(storage, rid, peer, &fetcher);
// (best-effort: ignore errors, log warnings)
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Identity repos are fetched after project repo fetch
- [ ] Failure to fetch identity repo does not fail the project fetch
- [ ] Already-local identity repos are updated (not re-cloned)
- [ ] No identity fetch attempted when project has no DID namespace refs
- [ ] `cargo test -p radicle-fetch` passes
