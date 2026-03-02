# Epic fn-3: SignedRefs Verification for Multi-Device

## Goal

Update the `SignedRefs` verification pipeline so that a signed ref authored by any
attested device of a KERI identity is treated as authorized by that identity. Also update
canonical reference aggregation so a multi-device identity has a single canonical view.

## Background

Currently `SignedRefs::verify()` checks that the signature matches exactly one `NodeId`
(the signer's own public key). For multi-device, a user may push from device B (with its
own `NodeId`) while the project doc lists the identity as `did:keri:<prefix>`. Verification
must:

1. Look up which KERI identity (if any) this `NodeId` is attested under.
2. Load the attestation chain and evaluate it against a policy (not-revoked, not-expired,
   has `sign_commit` capability by default).
3. Accept the signature if the attestation is valid.

The canonical reference view (`crefs.rs`) must also aggregate across all device namespaces
that belong to the same KERI identity.

## Affected Crates

- `crates/radicle/src/storage/refs.rs` — `SignedRefs::verify()` + multi-device path
- `crates/radicle/src/storage/git.rs` — `crefs` computation, namespace aggregation
- `crates/radicle/src/identity/crefs.rs` — `CanonicalRefs` aggregation logic
- `crates/radicle/src/storage/adapters.rs` — wire `HeartwooodAuthsStorage` into verifier
- `crates/radicle/src/node/device.rs` — `Device::verify_with_identity()` helper

## Key Design Decisions

1. **Layered verification**: The existing single-key path is untouched. A new
   `SignedRefs::verify_with_identity()` method tries the fast path first (exact key match),
   then falls back to attestation lookup if the signer is not a direct delegate.

2. **`DeviceAuthorization` struct** (new) wraps `DefaultBridge<HeartwooodAuthsStorage>` and
   exposes `fn is_authorized(node_id: &NodeId, repo_id: &RepoId, now: DateTime<Utc>) -> bool`.
   This is the single call site. All policy logic lives in `auths-radicle`.

3. **Time injection**: `DeviceAuthorization::is_authorized()` accepts `now: DateTime<Utc>`.
   Callers pass the timestamp from the signed ref commit to avoid clock-skew issues.

4. **Default policy**: `not_revoked() + not_expired() + require_capability("sign_commit")`.
   Policy is configurable via `NodeConfig` but this default ships out of the box.

5. **Canonical refs aggregation**: `CanonicalRefs` for a project aggregates all per-device
   namespaces `refs/namespaces/<nid>/...` whose `NodeId` is attested under the same KERI
   identity. The aggregation rule: latest commit wins per branch, ties broken by timestamp.

6. **Backwards compatibility**: Projects without KERI identity (pure `did:key`) continue
   working exactly as before. The attestation lookup short-circuits to `None` and the
   original key verification path handles it.

## Acceptance Criteria

- [ ] `DeviceAuthorization::is_authorized()` returns `true` for attested, non-expired device
- [ ] `DeviceAuthorization::is_authorized()` returns `false` for revoked device
- [ ] `DeviceAuthorization::is_authorized()` returns `false` for device with no attestation
- [ ] `SignedRefs::verify()` accepts refs signed by an attested device of a KERI identity
- [ ] `SignedRefs::verify()` rejects refs signed by a revoked device
- [ ] Pure `did:key` identities continue to verify as before (no regression)
- [ ] `CanonicalRefs` aggregates branches across all devices of a KERI identity
- [ ] `cargo test -p radicle` passes
