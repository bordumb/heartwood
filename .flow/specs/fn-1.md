# Epic fn-1: Core Identity Types — `did:keri` Support

## Goal

Extend Heartwood's identity type system to support the `did:keri:` DID method alongside
the existing `did:key:` method. Add the new reference namespace format for KERI identity
repos. Add `auths-radicle` as a dependency. No behavior changes — types and parsing only.

## Background

Currently `Did` wraps a single `PublicKey` and only encodes as `did:key:z6Mk...`. Per
RIP-X, we need to:
- Support `did:keri:<prefix>` as a DID method for multi-device identities
- Add git ref namespace: `refs/namespaces/did-keri-<prefix>/...` (`:` → `-`)
- Allow project `Doc` delegates to reference KERI DIDs, not just static keys
- Add `auths-radicle` as a path dependency so subsequent epics can use it

## Affected Crates

- `crates/radicle-crypto` — no changes
- `crates/radicle/src/identity/did.rs` — extend `Did` enum
- `crates/radicle/src/identity/doc.rs` — loosen delegate constraint
- `crates/radicle/src/storage/refs.rs` — new namespace pattern constants
- `crates/radicle/Cargo.toml` — add `auths-radicle` dependency
- `crates/radicle/src/lib.rs` — re-export new types

## Key Design Decisions

1. `Did` becomes an enum (`Key(PublicKey)` vs `Keri(String)`) rather than a newtype. The
   existing `Did::Key` variant preserves full backwards-compatibility.
2. The git ref namespace separator uses `-` not `:` because `:` is invalid in git refnames.
   The conversion is: `did:keri:<prefix>` → `did-keri-<prefix>`.
3. `auths-radicle` is added as a path dependency (not published crate) since both repos are
   local. This will need a published version or git dependency for production use.
4. No functional changes in this epic — only type definitions, parsing, and serialization.

## Acceptance Criteria

- [ ] `Did::Keri("did:keri:EXq5...")` parses from and serializes to `did:keri:EXq5...`
- [ ] `Did::Key(pk)` continues to work exactly as before (no regressions)
- [ ] `IdentityNamespace::from_did(Did::Keri(...))` produces correct ref prefix
- [ ] `Doc` delegates can hold either `Did::Key` or `Did::Keri` without panics
- [ ] `cargo test -p radicle` passes with no regressions
