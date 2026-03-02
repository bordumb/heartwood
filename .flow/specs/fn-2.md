# Epic fn-2: KERI Identity Repository Storage

## Goal

Implement the storage layer for KERI identity repos as described in RIP-X. A KERI identity
lives in its own git repo (with its own RID). This epic makes Heartwood able to create,
read, and write that repo, and teaches the storage layer how to interpret the DID namespace
pointer (`refs/rad/id`) in ordinary project repos.

## Background

Per RIP-X the KERI identity repo layout is:

```
<rid>
└── refs
    ├── keri
    │   └── kel                # KEL commit chain (via auths-id GitKel)
    └── keys
        └── <nid>              # per-device 2-way attestation
            └── signatures     # commit with did-key + did-keri blobs
```

In project repos, a new DID namespace is added:

```
refs/namespaces/did-keri-<prefix>/
└── refs
    └── rad
        └── id                 # Git blob: RID of the KERI identity repo
```

The `HeartwooodAuthsStorage` adapter bridges heartwood's git storage to the
`AuthsStorage` port defined in `auths-radicle`.

## Affected Crates

- `crates/radicle/src/identity/keri.rs` (new) — `KeriIdentityRepo` type
- `crates/radicle/src/identity/mod.rs` — re-export `KeriIdentityRepo`
- `crates/radicle/src/storage/refs.rs` — attestation ref constants
- `crates/radicle/src/storage/git.rs` — read/write DID namespace `refs/rad/id` blob
- `crates/radicle/src/storage/adapters.rs` (new) — `HeartwooodAuthsStorage` impl

## Key Design Decisions

1. **`KeriIdentityRepo` is a thin wrapper** around Heartwood's existing `git2::Repository`.
   It exposes typed accessors (`kel()`, `attested_devices()`, `attestation_for()`) without
   duplicating the KEL logic — that stays in `auths-id::keri::GitKel`.

2. **Ports-and-adapters boundary**: `AuthsStorage` (defined in `auths-radicle`) is the
   port. `HeartwooodAuthsStorage` is the adapter. The adapter holds a reference to
   Heartwood's `Storage<git2::Repository>` and translates calls:
   - `load_key_state(identity_did)` → opens the KERI repo by RID and calls `GitKel::get_key_state()`
   - `load_attestation(device_did)` → reads the `refs/keys/<nid>/signatures` tree
   - `find_identity_for_device(device_did)` → scans DID namespaces in known repos

3. **Attestation blobs follow RIP-X layout**:
   - `refs/keys/<nid>/signatures` → commit whose tree has two blobs:
     - `did-key` (bytes): identity's signature over `(RID, device_did_key)`
     - `did-keri` (bytes): device's signature over `(RID, identity_did_keri)`

4. **`refs/rad/id` blob** is a plain UTF-8 string holding the canonical RID, e.g.
   `rad:z42hL2jL4XNk6K8oHQaSWfMgCL7ji`. Written when a KERI identity is created, read
   during identity resolution.

## Acceptance Criteria

- [ ] `KeriIdentityRepo::open(storage, rid)` succeeds for a correctly initialized repo
- [ ] `KeriIdentityRepo::kel()` returns a `GitKel` pointing to `refs/keri/kel`
- [ ] `KeriIdentityRepo::attested_devices()` returns `Vec<NodeId>` from `refs/keys/`
- [ ] `HeartwooodAuthsStorage::load_key_state()` returns correct `KeyState` via GitKel
- [ ] `HeartwooodAuthsStorage::load_attestation()` deserializes 2-way attestation blobs
- [ ] DID namespace `refs/rad/id` blob round-trips (write → read → same RID)
- [ ] `cargo test -p radicle` passes with no regressions
