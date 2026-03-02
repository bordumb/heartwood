# fn-5.2: `rad identity device add` command

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## New File
`crates/radicle-cli/src/commands/identity/device.rs`

## What to Do

Add `rad identity device add [--key <node-id>]` to create a 2-way attestation between
the KERI identity and a device key.

### Two-Phase Flow (because both keys must sign)

#### Phase 1: Identity owner signs (on machine A)
```
$ rad identity device add --key z6MkNew...
→ Creates partial attestation (identity signature only)
→ Writes to a temp file: attestation-pending-z6MkNew.json
→ Prints: "Share attestation-pending-z6MkNew.json with the device owner."
```

#### Phase 2: Device co-signs (on machine B)
```
$ rad identity device add --co-sign attestation-pending-z6MkNew.json
→ Reads partial attestation
→ Adds device signature
→ Writes completed attestation to stdout (or file)
→ "Return the completed attestation to the identity owner."
```

#### Phase 3: Identity owner stores (on machine A)
```
$ rad identity device add --complete attestation-complete-z6MkNew.json
→ Verifies both signatures
→ Stores in identity repo: refs/keys/<nid>/signatures
→ Pushes identity repo
→ Prints: "Device z6MkNew... attested successfully."
```

### Alternative: Single-machine flow (both keys on same machine)
```
$ rad identity device add  # no --key flag = current node key
→ Creates and completes attestation in one step
→ Stores and pushes
```

### Command Structure

```rust
#[derive(Parser)]
pub enum DeviceAdd {
    /// Start attestation for another device (phase 1)
    #[command(name = "add")]
    Add { #[arg(long)] key: Option<NodeId> },

    /// Co-sign a pending attestation (phase 2, run on the device being added)
    #[command(name = "add --co-sign")]
    CoSign { file: PathBuf },

    /// Complete attestation with co-signed file (phase 3)
    #[command(name = "add --complete")]
    Complete { file: PathBuf },
}
```


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Single-machine flow creates valid 2-way attestation and stores it
- [ ] Two-phase flow correctly splits into partial + complete attestation
- [ ] Stored attestation passes `verify_chain()` from `auths-verifier`
- [ ] Clear error if KERI identity doesn't exist (`suggest: rad identity init-keri`)
