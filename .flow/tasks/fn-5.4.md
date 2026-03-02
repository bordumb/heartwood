# fn-5.4: `rad identity device revoke` command

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle-cli/src/commands/identity/device.rs`

## What to Do

Add `rad identity device revoke <node-id>` to permanently revoke an attested device.

### Flow

1. Load the existing attestation for `<node-id>`
2. Set `revoked_at = Utc::now()`
3. Re-sign with the identity key (`auths-id::attestation::resign_attestation()`)
4. Store the updated attestation via `GitKeriIdentityStore::store_attestation()`
5. Push identity repo to network
6. Print: `✓ Device <nid> revoked at <timestamp>`

### Important: Revocation is permanent

Once `revoked_at` is set, the `CompositeAuthorityChecker` will reject any future `SignedRefs`
from that device (even if the refs were created before revocation). This is the intended
behavior per the RIP — revocation is not retroactive in history but is enforced going forward.

### Confirmation Prompt

```
Are you sure you want to revoke device z6MknSLr...?
This action cannot be undone. [y/N]:
```

Use `radicle-term`'s prompt utilities.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Sets `revoked_at` on attestation and re-signs with identity key
- [ ] Confirmation prompt before revoking
- [ ] Error if node-id has no attestation
- [ ] After revocation, `device list` shows the device as REVOKED
- [ ] After revocation, `verify_with_authority()` rejects refs from that device
