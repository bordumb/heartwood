# fn-4.4: Propagate DID namespace refs in `RefsAnnouncement`

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## Files
- `crates/radicle-protocol/src/service/message.rs`
- `crates/radicle-protocol/src/service.rs`

## What to Do

DID namespace refs (`refs/namespaces/did-keri-*/refs/rad/id`) should be included in
`RefsAnnouncement` so that remote nodes can discover which identity repos to fetch.

### Analysis First

Read the existing `RefsAnnouncement` and `RefsAt` types carefully. Determine whether DID
namespace refs are already included in the announcement naturally (since they're just refs
under `refs/namespaces/`) or whether they're being filtered out.

Check `SIGREFS_GLOB` and related patterns in `git.rs` to see if DID namespace refs are
excluded.

### Option A: Already included (preferred)

If `refs/namespaces/did-keri-*/refs/rad/id` refs are already naturally included in the
announcement (they're just git refs), then this task is mainly:
- Update namespace filtering to NOT exclude DID namespace refs
- Add a test verifying DID namespace refs appear in announcements

### Option B: Need to add them

If they're being filtered, add them to the set of refs included in `RefsAnnouncement`.
Do this additively — do not remove any existing refs.

### Key Constraint: No Breaking Changes

The protocol format must not change in a way that breaks old nodes. DID namespace refs are
just extra refs — old nodes will attempt to fetch them and may fail, but that should be
handled gracefully.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] `refs/namespaces/did-keri-*/refs/rad/id` refs appear in `RefsAnnouncement`
- [ ] Old-format nodes receiving announcements with DID namespace refs handle gracefully
- [ ] `cargo test -p radicle-protocol` passes
