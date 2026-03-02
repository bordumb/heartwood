# fn-1.5: Add `auths-radicle` as dependency to `radicle` crate

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/Cargo.toml`

## What to Do

Add `auths-radicle` (and transitively `auths-verifier`, `auths-id`) as a dependency so
subsequent epics can use the bridge types.

```toml
[dependencies]
# ... existing deps ...

# Multi-device identity bridge
auths-radicle = { path = "/Users/bordumb/workspace/repositories/auths-base/auths/crates/auths-radicle" }
```

### Notes

- Use a `path =` dependency for now since both repos are local.
- For production/CI, this needs to become a published crate version or git dependency.
  Document this in a `# TODO: publish auths-radicle` comment in Cargo.toml.
- `auths-radicle` pulls in `auths-id` and `auths-verifier` transitively. No need to add
  them directly.
- Verify there are no conflicting dependency versions (particularly `chrono`, `serde`,
  `git2`) — check `cargo tree -p radicle` for version conflicts and resolve if needed.

### Potential Conflicts to Watch

| Dep | heartwood version | auths-radicle version |
|-----|-------------------|-----------------------|
| `git2` | 0.19 | 0.19 | (same ✓) |
| `serde` | 1 | 1 | (same ✓) |
| `chrono` | check | 0.4 | |
| `ed25519` family | check | ed25519-dalek 2.1.1 | |

If `ed25519-dalek` conflicts with heartwood's `ec25519` usage, the bridge types can be
kept in a `feature = ["multi-device"]` gate so the dependency is opt-in.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [x] `cargo build -p radicle` succeeds after adding the dependency
- [x] No duplicate/conflicting crates at different versions (check `cargo tree`)
- [ ] `auths_radicle::RadicleAuthsBridge` is importable from `radicle` tests

## Done summary
- Extended `Did` from a newtype to an enum with `Key(PublicKey)` and `Keri(String)` variants
- Added `FromStr`/`Display` round-trip for both `did:key:` and `did:keri:` formats
- Added helper methods: `as_key()`, `as_keri_prefix()`, `is_rotatable()`, `to_ref_component()`
- Why: Foundation for multi-device identity support via KERI DIDs
- Verification: `cargo test -p radicle -- identity::did` passes (4 tests)
## Evidence
- Commits:
- Tests: cargo test -p radicle
- PRs: