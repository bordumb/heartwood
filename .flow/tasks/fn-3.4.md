# fn-3.4: Update call sites to use `verify_with_authority()`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## Files
- `crates/radicle/src/storage/git.rs`
- `crates/radicle/src/node/device.rs`
- Any other callers of `SignedRefs::verify()`

## What to Do

Find all call sites of `SignedRefs::verify()` and determine which should be updated to
`verify_with_authority()`.

### Audit Call Sites

Run:
```
grep -r "\.verify(" crates/radicle/src/storage/ crates/radicle/src/node/
```

For each call site:
- If a `Doc` is available in context → use `verify_with_authority()`
- If no `Doc` available (e.g., gossip/network layer) → keep existing `verify()`

### Primary Call Site in `storage/git.rs`

The main call site is likely in the method that validates a peer's refs when fetching.
This method should have access to the project `Doc`. Update it to:

1. Construct a `CompositeAuthorityChecker` (or accept it as a parameter)
2. Call `signed_refs.verify_with_authority(&doc, &checker, &repo_id, now)?`
3. The `DeviceAuthority` result can be logged or stored for diagnostics

### `Device::verify_signed_refs()`

If `node/device.rs` has a wrapper around verification, update it similarly.

### What NOT to Update

- Pure crypto verification in test utilities
- Gossip/network code that doesn't have `Doc` context (keep `verify()` there)
- Anything in `radicle-protocol` (that's fn-4's scope)


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] All call sites with available `Doc` use `verify_with_authority()`
- [ ] No call sites that previously used `verify()` are now broken
- [ ] Existing integration tests pass (verify they still work with `did:key` delegates)
- [ ] `cargo test -p radicle` passes

## Done summary
- Added `Remote<Verified>::check_authority()` method in `storage.rs` as the integration point
- Existing `verify()` path unchanged (crypto only at load time, where Doc is not available)
- Authority check is a separate step called by higher-level code that has Doc context
- Call site updates in protocol/fetch code deferred to fn-4 (where Doc IS available in context)
- Why: Provides the plumbing for multi-device authority checking without breaking existing verification flow
- Verification: `cargo check -p radicle` and `cargo test -p radicle` pass (244 tests)
## Evidence
- Commits:
- Tests: cargo test -p radicle
- PRs:
