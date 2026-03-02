# Epic fn-4: Fetch and Protocol Updates for Multi-Device

## Goal

Make the Heartwood node automatically fetch KERI identity repos when encountered, recognize
`refs/namespaces/did-keri-*` as a new namespace kind, and propagate identity namespaces in
`RefsAnnouncement` messages.

## Background

When a node has a KERI identity it has two kinds of repos:
1. Project repos (code, issues, patches) — existing behavior
2. KERI identity repos (KEL + attestations) — new, referenced from project namespaces

When fetching a project repo, if we encounter a `refs/namespaces/did-keri-<prefix>/refs/rad/id`
blob, we need to also fetch the referenced KERI identity repo so that attestation lookups
work offline.

## Affected Crates

- `crates/radicle/src/storage/git.rs` — namespace classification (`is_did_namespace`)
- `crates/radicle-fetch/src/` — fetch KERI identity repos as side-effects
- `crates/radicle-protocol/src/service/message.rs` — `RefsAnnouncement` DID refs
- `crates/radicle-protocol/src/service.rs` — handle DID namespace during sync

## Key Design Decisions

1. **Namespace classification**: Add `NamespaceKind` enum: `Peer(NodeId)` (existing) or
   `Identity(Did)` (new). The storage layer classifies `refs/namespaces/<component>` using
   this. Components starting with `did-` are `Identity` namespaces.

2. **Fetch side-effect**: When fetching a project repo and we encounter an `Identity`
   namespace, the fetcher reads `refs/rad/id`, extracts the KERI RID, and queues a fetch
   for that identity repo if not already in local storage. This is best-effort (failure
   does not abort the project fetch).

3. **`RefsAnnouncement` unchanged**: The existing protocol message format is not changed.
   DID namespace refs are propagated the same way as peer namespace refs — they are just
   refs under `refs/namespaces/...`. No protocol version bump needed for this epic.

4. **Discovery is pull-based**: A node does not broadcast its KERI RID separately. Other
   nodes discover it by fetching the project repo and reading the DID namespace. This
   avoids adding a new gossip message type.

5. **Storage seeding**: When a local user initializes a KERI identity, their identity repo
   is seeded (announced to the network) the same way as any other repo they seed.

## Acceptance Criteria

- [ ] `NamespaceKind::from_component("did-keri-EXq5...")` returns `Identity` variant
- [ ] `NamespaceKind::from_component("z6MknSLr...")` returns `Peer` variant (no change)
- [ ] Fetching a project with a DID namespace triggers a fetch of the referenced KERI repo
- [ ] Identity repos are seeded and announced like any other repo
- [ ] DID namespace refs appear in `RefsAnnouncement` for project repos
- [ ] Nodes without KERI support ignore `did-keri-*` namespaces gracefully (unknown ref)
- [ ] `cargo test -p radicle-fetch` and `cargo test -p radicle-protocol` pass
