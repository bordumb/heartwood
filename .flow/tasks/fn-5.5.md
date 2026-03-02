# fn-5.5: `rad identity rotate` command

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle-cli/src/commands/identity/keri.rs`

## What to Do

Add `rad identity rotate [--next-key <pubkey>]` to rotate the KERI identity key by
appending a rotation event to the KEL.

### Prerequisites

KERI key rotation requires a **pre-commitment**: at inception (or last rotation), you
committed to a hash of the next key. Rotation is only possible if you have the key whose
hash matches the `next_commitment` in the current `KeyState`.

### Flow

1. Load current `KeyState` via `GitKeriIdentityStore::key_state()`
2. Verify the user has the pre-committed key (prompt for it or load from keystore)
3. Optionally generate a new next key (for future rotations)
4. Call `auths_id::keri::rotate_keys(current_state, new_keys, new_next_commitments)`
5. Append `RotEvent` to `refs/keri/kel` via `GitKeriIdentityStore`
6. Update all existing device attestations to reflect new identity key (re-sign)
7. Push identity repo
8. Print: `✓ Identity key rotated. New key: <pubkey>`

### Note on Device Attestations

After rotation, the identity signature on all device attestations must be refreshed with
the new key. This is done automatically as part of the rotation flow.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Appends a valid `RotEvent` to the KEL
- [ ] `key_state()` after rotation shows the new current key
- [ ] Rotation fails gracefully if pre-commitment doesn't match
- [ ] Rotation fails if identity is already abandoned (empty next commitment)
- [ ] Device attestations are re-signed with new key after rotation
