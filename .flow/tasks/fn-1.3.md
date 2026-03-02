# fn-1.3: Add `IdentityNamespace` type for DID-to-ref-name conversion

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## New File
`crates/radicle/src/identity/namespace.rs`

## What to Do

Centralise all DID ↔ git-refname conversions in one type so the conversion rule
(`:` → `-`) is never duplicated.

```rust
/// A git ref namespace component derived from a DID.
///
/// Per RIP-X: `did:keri:<prefix>` → `did-keri-<prefix>`
///            `did:key:<pubkey>`  → `did-key-<pubkey>`  (for consistency)
pub struct IdentityNamespace {
    did: Did,
}

impl IdentityNamespace {
    pub fn new(did: Did) -> Self

    /// Returns the git ref component, e.g. "did-keri-EXq5..."
    pub fn ref_component(&self) -> String

    /// Returns the full namespace prefix, e.g. "refs/namespaces/did-keri-EXq5..."
    pub fn ref_prefix(&self) -> String

    /// Returns "refs/namespaces/did-keri-<prefix>/refs/rad/id"
    pub fn rad_id_ref(&self) -> String

    /// Parse a ref component back into a DID namespace, if it matches the pattern.
    /// Returns `None` for ordinary peer NIDs.
    pub fn from_ref_component(component: &str) -> Option<Self>
}
```

### Pattern

The ref component for a KERI DID:
- Input:  `did:keri:EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148`
- Output: `did-keri-EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148`

Detection: a ref component starting with `did-` is an identity namespace.

### Re-export

Export from `crates/radicle/src/identity/mod.rs`:
```rust
pub use namespace::IdentityNamespace;
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [x] `IdentityNamespace::from_ref_component("did-keri-EXq5...")` returns `Some`
- [x] `IdentityNamespace::from_ref_component("z6MknSLr...")` returns `None`
- [x] `.ref_component()` and `.from_ref_component()` round-trip correctly
- [x] `.rad_id_ref()` returns the correct full refname
- [x] Unit tests in `namespace.rs`

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