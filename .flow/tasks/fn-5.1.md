# fn-5.1: `rad identity init-keri` command

## Repo
`heartwood` — `/Users/bordumb/workspace/repositories/heartwood`

## New File
`crates/radicle-cli/src/commands/identity/keri.rs`

## What to Do
## Action: SDK Embedding
**STRICT**: The `rad identity init-keri` command MUST be a thin wrapper.
1. DO NOT reimplement key generation or KEL serialization in Heartwood.
2. Call `auths_sdk::keys::import_seed` (from Auths fn-3.4) directly.
3. Use the `auths_radicle::refs` helpers to ensure the repository is initialized with the correct RIP-X layout.


Add a subcommand `rad identity init-keri` that bootstraps a new KERI identity.

### Flow

1. Generate a new Ed25519 key pair (inception key) via `auths-id`'s `create_keri_identity()`
2. Optionally prompt for a pre-committed rotation key (or generate one)
3. Create a new bare git repo in Heartwood storage
4. Initialize `GitKeriIdentityStore` on that repo
5. Write the inception event to `refs/keri/kel`
6. Seed the repo (announce it to the network)
7. Print:
   ```
   ✓ KERI identity created
   DID:    did:keri:EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148
   RID:    rad:z42hL2jL4XNk6K8oHQaSWfMgCL7ji
   ```

### Command Structure

```rust
#[derive(Parser)]
pub struct InitKeri {
    /// Pre-committed rotation key (hex). If omitted, one is generated for you.
    #[arg(long)]
    next_key: Option<String>,
}
```

### File to Hook Into

Find where other `rad identity` subcommands are registered and add `keri` there.


## Code Quality
- **DRY**: Do not repeat logic; extract shared patterns into helper functions or traits.
- **Modular Design**: Avoid monolithic functions. Decompose complex logic into small, focused, and testable units.
- **Strictness**: Adhere to the "Zero-Debt" mandate—if old code is redundant, delete it; do not leave "TODO" or "Legacy" stubs.

## Acceptance Criteria

- [ ] Command creates a valid KERI identity repo with inception event
- [ ] DID and RID printed to stdout
- [ ] Error if KERI identity already exists for this node
- [ ] `cargo test -p radicle-cli -- commands::identity::keri` passes
