# KERI Identity at `rad auth` — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make every radicle identity start as `did:keri` by creating a KERI inception event during `rad auth`, registering the Ed25519 key as the first device.

**Architecture:** Add a `create_keri_identity_from_key()` function to auths-id that accepts an existing Ed25519 keypair (instead of generating one). During `Profile::init()`, after creating the Ed25519 key as today, call this function to create a KERI inception event. Store the KERI prefix in `$RAD_HOME/keys/keri-prefix`. Change `Profile::did()` to return `Did::Keri(prefix)` when a prefix file exists, falling back to `Did::Key` for pre-existing profiles.

**Tech Stack:** Rust, auths-id (KERI inception), radicle (profile/keystore), ring (Ed25519)

---

### Task 1: Add `create_keri_identity_from_key()` to auths-id

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/auths-base/auths/crates/auths-id/src/keri/inception.rs`

**Step 1: Add the new function**

Add this function after `create_keri_identity()` (after line 188):

```rust
/// Create a KERI identity using an existing Ed25519 keypair as the current key.
///
/// Unlike [`create_keri_identity`], this does NOT generate the current keypair.
/// The caller provides a PKCS8-encoded Ed25519 key (e.g. the radicle device key).
/// A new next-rotation keypair IS generated internally.
///
/// This enables radicle's `rad auth` to create a KERI identity that wraps
/// the existing radicle Ed25519 key as the first device.
pub fn create_keri_identity_from_key(
    repo: &Repository,
    current_keypair_pkcs8: &[u8],
    witness_config: Option<&WitnessConfig>,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<InceptionResult, InceptionError> {
    let rng = SystemRandom::new();

    // Use the provided keypair as current
    let current_keypair = Ed25519KeyPair::from_pkcs8(current_keypair_pkcs8)
        .map_err(|e| InceptionError::KeyGeneration(e.to_string()))?;

    // Generate next keypair (for pre-rotation) — this one IS new
    let next_pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|e| InceptionError::KeyGeneration(e.to_string()))?;
    let next_keypair = Ed25519KeyPair::from_pkcs8(next_pkcs8.as_ref())
        .map_err(|e| InceptionError::KeyGeneration(e.to_string()))?;

    // Encode current public key with 'D' derivation code (Ed25519)
    let current_pub_encoded = format!(
        "D{}",
        URL_SAFE_NO_PAD.encode(current_keypair.public_key().as_ref())
    );

    // Compute next-key commitment (Blake3 hash of next public key)
    let next_commitment = compute_next_commitment(next_keypair.public_key().as_ref());

    // Build inception event
    let icp = IcpEvent {
        v: KERI_VERSION.to_string(),
        d: Said::default(),
        i: Prefix::default(),
        s: KeriSequence::new(0),
        kt: "1".to_string(),
        k: vec![current_pub_encoded],
        nt: "1".to_string(),
        n: vec![next_commitment],
        bt: "0".to_string(),
        b: vec![],
        a: vec![],
        x: String::new(),
    };

    // Finalize (computes SAID), sign, store — same as create_keri_identity
    let mut finalized = finalize_icp_event(icp)?;
    let prefix = finalized.i.clone();

    let canonical = super::serialize_for_signing(&Event::Icp(finalized.clone()))?;
    let sig = current_keypair.sign(&canonical);
    finalized.x = URL_SAFE_NO_PAD.encode(sig.as_ref());

    let kel = GitKel::new(repo, prefix.as_str());
    kel.create(&finalized, now)?;

    Ok(InceptionResult {
        prefix,
        current_keypair_pkcs8: Pkcs8Der::new(current_keypair_pkcs8),
        next_keypair_pkcs8: Pkcs8Der::new(next_pkcs8.as_ref()),
        current_public_key: current_keypair.public_key().as_ref().to_vec(),
        next_public_key: next_keypair.public_key().as_ref().to_vec(),
    })
}
```

**Step 2: Export the new function**

In the same file's module or parent `mod.rs`, ensure `create_keri_identity_from_key` is re-exported alongside `create_keri_identity`.

Check `/Users/bordumb/workspace/repositories/auths-base/auths/crates/auths-id/src/keri/mod.rs` — add `create_keri_identity_from_key` to the `pub use inception::` line.

**Step 3: Verify it compiles**

Run: `cd /Users/bordumb/workspace/repositories/auths-base/auths && cargo check -p auths-id`
Expected: compiles with no errors

**Step 4: Commit**

```bash
git add crates/auths-id/src/keri/inception.rs crates/auths-id/src/keri/mod.rs
git commit -m "feat: add create_keri_identity_from_key() for existing Ed25519 keys"
```

---

### Task 2: Store and load KERI prefix in radicle profile

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/profile.rs`

**Step 1: Add KERI prefix storage helpers to `Home`**

After the `keys()` method (line ~589), add:

```rust
    /// Path to the KERI prefix file.
    /// Contains the KERI prefix string when this profile has a KERI identity.
    pub fn keri_prefix(&self) -> PathBuf {
        self.keys().join("keri-prefix")
    }
```

**Step 2: Add KERI prefix storage helpers to `Profile`**

After the `did()` method (line ~322), add:

```rust
    /// Load the KERI prefix from disk, if it exists.
    pub fn keri_prefix(&self) -> Option<String> {
        let path = self.home.keri_prefix();
        std::fs::read_to_string(&path).ok().map(|s| s.trim().to_string())
    }
```

**Step 3: Change `Profile::did()` to return `Did::Keri` when prefix exists**

Replace the existing `did()` method:

```rust
    pub fn did(&self) -> Did {
        if let Some(prefix) = self.keri_prefix() {
            Did::Keri(prefix)
        } else {
            Did::from(self.public_key)
        }
    }
```

**Step 4: Verify it compiles**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo check -p radicle`
Expected: compiles (existing profiles without keri-prefix file fall back to Did::Key)

**Step 5: Commit**

```bash
git add crates/radicle/src/profile.rs
git commit -m "feat: Profile::did() returns Did::Keri when KERI prefix exists"
```

---

### Task 3: Create KERI identity during `Profile::init()`

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/profile.rs`

**Step 1: Add auths-id import and KERI inception call**

At the top of `profile.rs`, add:

```rust
use auths_id::keri::inception::create_keri_identity_from_key;
```

In `Profile::init()`, after the keystore init (line ~239) and before config init (line ~240), add the KERI inception:

```rust
    pub fn init(
        home: Home,
        alias: Alias,
        passphrase: Option<Passphrase>,
        seed: crypto::Seed,
    ) -> Result<Self, Error> {
        let keystore = Keystore::new(&home.keys());
        let public_key = keystore.init("radicle", passphrase, seed)?;

        // Create KERI identity using the radicle key as the first device.
        // The KEL is stored in a git repo under $RAD_HOME/keys/keri/
        // and the prefix is persisted to $RAD_HOME/keys/keri-prefix.
        let keri_repo_path = home.keys().join("keri");
        let keri_repo = if keri_repo_path.exists() {
            git2::Repository::open(&keri_repo_path)
        } else {
            git2::Repository::init(&keri_repo_path)
        }.map_err(|e| Error::Init(e.into()))?;

        let secret_key_pkcs8 = keystore.secret_key_pkcs8(passphrase.clone())
            .map_err(|e| Error::Init(e.into()))?;

        let inception = create_keri_identity_from_key(
            &keri_repo,
            &secret_key_pkcs8,
            None, // no witnesses for now
            chrono::Utc::now(),
        ).map_err(|e| Error::Init(e.into()))?;

        // Persist the KERI prefix
        std::fs::write(home.keri_prefix(), inception.prefix.as_str())
            .map_err(|e| Error::Io(e))?;

        // Store next-rotation key securely
        let next_key_path = home.keys().join("keri-next");
        std::fs::write(&next_key_path, inception.next_keypair_pkcs8.as_ref())
            .map_err(|e| Error::Io(e))?;

        // ... rest of init continues unchanged ...
```

**Important:** The exact integration depends on how `Keystore` exposes the PKCS8 bytes. We may need to add a `secret_key_pkcs8()` method to the keystore (see Task 4).

**Step 2: Add `Error::Init` variant if needed**

Check if the `Error` enum in `profile.rs` has a generic init variant. If not, add:

```rust
    #[error("initialization failed: {0}")]
    Init(Box<dyn std::error::Error + Send + Sync>),
```

**Step 3: Verify it compiles**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo check -p radicle`
Expected: may fail if `secret_key_pkcs8` doesn't exist yet — that's Task 4

**Step 4: Commit**

```bash
git add crates/radicle/src/profile.rs
git commit -m "feat: create KERI identity during Profile::init()"
```

---

### Task 4: Expose PKCS8 key bytes from radicle Keystore

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/crypto/ssh/keystore.rs`

The radicle `Keystore` stores keys in OpenSSH format. We need to extract the Ed25519 seed bytes (32 bytes) and wrap them as PKCS8 for the auths-id inception API.

**Step 1: Find the keystore implementation**

Check `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/crypto/ssh/keystore.rs` or wherever `Keystore::init` is defined. Understand how the private key is stored and loaded.

**Step 2: Add a method to get raw Ed25519 seed bytes**

Add to `Keystore`:

```rust
    /// Extract the Ed25519 private key seed as raw bytes.
    /// Returns the 32-byte seed that can be used to reconstruct the keypair.
    pub fn secret_key_seed(&self, passphrase: Option<Passphrase>) -> Result<[u8; 32], Error> {
        let secret = self.secret_key(passphrase)?
            .ok_or(Error::NotFound)?;
        // Extract Ed25519 seed from the ssh_key::PrivateKey
        // The exact extraction depends on the ssh_key crate API
        let ed25519_key = secret.key_data().ed25519()
            .ok_or_else(|| Error::Other("not an Ed25519 key".into()))?;
        let mut seed = [0u8; 32];
        seed.copy_from_slice(ed25519_key.private.as_ref());
        Ok(seed)
    }
```

**Step 3: Build PKCS8 from seed in Profile::init()**

In `Profile::init()`, instead of calling a `secret_key_pkcs8()` method, construct PKCS8 from the seed:

```rust
    let seed_bytes = keystore.secret_key_seed(passphrase.clone())
        .map_err(|e| Error::Init(e.into()))?;

    // Wrap Ed25519 seed as PKCS8 DER for auths-id
    let pkcs8 = ring::signature::Ed25519KeyPair::from_seed_unchecked(&seed_bytes)
        .map_err(|e| Error::Init(e.into()))?;
    // Note: ring doesn't directly give PKCS8 from seed. Alternative approach:
    // use the ring PKCS8 generation and verify the public key matches.
```

**Alternative approach (simpler):** Since `Keystore::init()` generates the key using `ring`, capture the PKCS8 bytes at generation time and pass them to the KERI inception, rather than extracting them after the fact.

This may require modifying `Keystore::init()` to return the PKCS8 bytes alongside the public key.

**Step 4: Verify it compiles**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo check -p radicle`

**Step 5: Commit**

```bash
git add crates/radicle/src/crypto/ssh/keystore.rs crates/radicle/src/profile.rs
git commit -m "feat: expose Ed25519 seed from Keystore for KERI inception"
```

---

### Task 5: Update `rad auth` CLI output

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle-cli/src/commands/auth.rs`

**Step 1: Update the success message**

Change line 82-86 from:

```rust
    term::success!(
        "Your Radicle DID is {}. This identifies your device. Run {} to show it at all times.",
        term::format::highlight(profile.did()),
        term::format::command("rad self")
    );
```

To:

```rust
    term::success!(
        "Your Radicle DID is {}.",
        term::format::highlight(profile.did()),
    );
    term::success!(
        "Your device key is {}.",
        term::format::highlight(Did::from(profile.public_key)),
    );
    term::success!("You're all set.");
```

**Step 2: Update the spinner text**

Change line 59 from:

```rust
    let spinner = term::spinner("Creating your Ed25519 keypair...");
```

To:

```rust
    let spinner = term::spinner("Creating your identity and device keypair...");
```

**Step 3: Verify it compiles**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo check -p radicle-cli`

**Step 4: Commit**

```bash
git add crates/radicle-cli/src/commands/auth.rs
git commit -m "feat: update rad auth output to show KERI DID and device key"
```

---

### Task 6: Update `rad self` to show KERI identity

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle-cli/src/commands/self.rs` (or wherever `rad self` is implemented)

**Step 1: Find the `rad self` implementation**

Search for the command that outputs the DID/alias info shown in `rad self`.

**Step 2: Add KERI prefix display**

After the DID line, if the profile has a KERI prefix, show it:

```rust
    // Existing:
    table.push("DID", term::format::tertiary(profile.did()));

    // Add device key line when identity is KERI:
    if profile.keri_prefix().is_some() {
        table.push("Device", term::format::tertiary(Did::from(profile.public_key)));
    }
```

**Step 3: Verify and commit**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo check -p radicle-cli`

```bash
git add crates/radicle-cli/src/commands/self.rs
git commit -m "feat: rad self shows device key when identity is KERI"
```

---

### Task 7: Integration test — fresh `rad auth` creates KERI identity

**Files:**
- Create: a test or manual verification

**Step 1: Manual test**

```bash
# Wipe existing profile (CAREFUL — only in test environment)
rm -rf /tmp/test-radicle-home

# Run rad auth with test home
RAD_HOME=/tmp/test-radicle-home RAD_PASSPHRASE="" rad auth --alias test-keri

# Verify KERI prefix was created
cat /tmp/test-radicle-home/keys/keri-prefix
# Expected: E<base64 SAID> (KERI prefix string)

# Verify DID is did:keri
RAD_HOME=/tmp/test-radicle-home rad self
# Expected: DID = did:keri:E...
# Expected: Device = did:key:z6Mk...

# Verify KEL exists
ls /tmp/test-radicle-home/keys/keri/
# Expected: git repo with KERI inception event
```

**Step 2: Commit any test fixtures**

```bash
git commit -m "test: verify rad auth creates KERI identity"
```

---

### Task 8: Backward compatibility — existing profiles keep `did:key`

**No code changes needed.** This is a verification task.

**Step 1: Verify existing profile still works**

```bash
# With your existing ~/.radicle profile (no keri-prefix file):
rad self
# Expected: DID = did:key:z6Mk... (unchanged)
# No "Device" line shown
```

The `Profile::did()` fallback to `Did::Key` when no `keri-prefix` file exists handles this automatically.

---

## Dependency Notes

- **auths-id** must be compiled with `git-storage` feature (for `git2::Repository` in inception)
- **radicle crate** already depends on `auths-id` via the auths integration
- **ring** is already a dependency of both codebases
- **chrono** is already a dependency (needed for `Utc::now()` in inception)
- **git2** is already a dependency of radicle (and auths-id with `git-storage` feature)

## Risk Areas

1. **PKCS8 key extraction** — The radicle keystore uses OpenSSH format internally. Extracting the raw Ed25519 seed to pass to `ring::Ed25519KeyPair` requires careful handling. Task 4 is the trickiest.
2. **Key format mismatch** — `ring` generates PKCS8-wrapped keys, `ssh_key` uses OpenSSH format. The 32-byte Ed25519 seed is the same in both, but extraction paths differ.
3. **Existing profile migration** — NOT in scope. Existing `did:key` profiles continue to work. Migration to `did:keri` would be a separate feature.
