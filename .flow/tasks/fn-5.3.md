# fn-5.3: `rad identity device list` command

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## File
`crates/radicle-cli/src/commands/identity/device.rs`

## What to Do

Add `rad identity device list` to show all devices attested under the user's KERI identity.

### Output Format

```
Device NodeId                                   Status   Expires              Capabilities
z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi  active   2026-06-01          sign_commit
z6Mkr84BHkMhm3dBWzckzS6P46sGFxGAFRWgjCcL5KqCdBFk  active   never               sign_commit, sign_release
z6MkiWkBShPE3mMWpBvXFG7TZCW7W3GK1c2t8HZEpHmZjNPi  REVOKED  (revoked 2026-01-15)
```

### Implementation

```rust
pub fn run_device_list(store: &GitKeriIdentityStore) -> Result<(), Error> {
    let devices = store.list_devices()?;
    for nid in devices {
        let att = store.load_attestation(&nid)?;
        let status = if att.revoked_at.is_some() { "REVOKED" } else { "active" };
        let expires = att.expires_at.map(|e| e.to_string()).unwrap_or("never".to_string());
        let caps = att.capabilities.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", ");
        println!("{}  {}  {}  {}", nid, status, expires, caps);
    }
    Ok(())
}
```

Use `radicle-term` table formatting to match existing `rad` command UX.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Lists all attested devices with correct status
- [ ] Revoked devices shown with REVOKED indicator
- [ ] Empty list (no error) when no devices have been attested
- [ ] Output uses radicle-term table formatting
