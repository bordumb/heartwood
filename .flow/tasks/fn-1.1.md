# fn-1.1: Extend `Did` enum to support `did:keri:` DID method

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle/src/identity/did.rs`

## What to Do

Change `Did` from a newtype over `PublicKey` to an enum:

```rust
pub enum Did {
    Key(PublicKey),          // did:key:z6Mk...
    Keri(String),            // did:keri:<keri-prefix>
}
```

### Encoding / Decoding

- `Did::Key(pk)` encodes as `did:key:<multibase(pk)>` (unchanged)
- `Did::Keri(prefix)` encodes as `did:keri:<prefix>`
- `FromStr` / `Display` must round-trip correctly for both variants
- JSON / CBOR serialization: serialize as string, same format

### `did:keri:` Validation

- The prefix must be non-empty
- Validate the prefix is a valid KERI self-addressing identifier (SAI) — base-58 or
  base-64url encoded, depending on derivation code. For now, accept any non-empty
  alphanumeric-with-dashes string (full KERI prefix validation can land separately).

### Helper Methods

```rust
impl Did {
    pub fn as_key(&self) -> Option<&PublicKey> { ... }
    pub fn as_keri_prefix(&self) -> Option<&str> { ... }
    pub fn is_rotatable(&self) -> bool { matches!(self, Did::Keri(_)) }
    pub fn to_ref_component(&self) -> String {
        // did:key:z6Mk... → "z6Mk..."  (existing behavior for Key)
        // did:keri:EXq5... → "did-keri-EXq5..."
    }
}
```

### Backwards Compatibility

- `From<PublicKey> for Did` → `Did::Key(pk)` (keep working)
- All existing code using `did.as_key().unwrap()` continues compiling (just add unwrap
  or match arms where compiler demands it)


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [x] `"did:keri:EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148".parse::<Did>()` succeeds
- [x] `"did:key:z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi".parse::<Did>()` succeeds
- [x] Both round-trip through `to_string()` + `from_str()`
- [x] `cargo test -p radicle -- identity::did` passes

## Done summary
- Extended `Did` from a newtype to an enum with `Key(PublicKey)` and `Keri(String)` variants
- Added `FromStr`/`Display` round-trip for both `did:key:` and `did:keri:` formats
- Added helper methods: `as_key()`, `as_keri_prefix()`, `is_rotatable()`, `to_ref_component()`
- Why: Foundation for multi-device identity support via KERI DIDs
- Verification: `cargo test -p radicle -- identity::did` passes (4 tests)
## Evidence
- Commits: 5fe4ffae48cc53fd520747a379da5634731c0f48
- Tests: cargo test -p radicle -- identity::did
- PRs: