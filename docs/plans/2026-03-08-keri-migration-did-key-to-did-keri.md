# Silent did:key → did:keri Migration Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Automatically migrate existing radicle profiles from `did:key` to `did:keri` on next profile load — zero user friction.

**Architecture:** When `Profile::load()` detects a profile with keys but no `keri-prefix` file, it reconstructs PKCS8 bytes from the existing Ed25519 OpenSSH secret key, runs KERI inception, and writes the prefix/next-key files. Encrypted keys require a passphrase, so migration defers to a helper that's called both from `load()` (best-effort with env passphrase) and from `rad auth` (where user provides passphrase). If migration fails for any reason, the profile still loads as `did:key` — no breakage.

**Tech Stack:** Rust, `radicle-crypto` (Ed25519/OpenSSH keystore), `auths-id` (KERI inception), `git2` (KEL repo)

---

### Task 1: Extract PKCS8 reconstruction into a reusable function on `Keystore`

The PKCS8 v2 DER encoding logic exists in `Keystore::init()` (lines 98-131 of `keystore.rs`). We need the same logic available for existing keys, not just newly generated ones. Add a method `Keystore::pkcs8_from_secret_key()` that takes a `SecretKey` and returns the 83-byte PKCS8 v2 DER.

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle-crypto/src/ssh/keystore.rs`

**Step 1: Write the failing test**

Add to the bottom of `keystore.rs` (or in an existing test module if there is one):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Seed;

    #[test]
    fn test_pkcs8_from_secret_key_matches_init() {
        let tmp = tempfile::tempdir().unwrap();
        let keystore = Keystore::new(&tmp.path());
        let seed = Seed::generate();
        let (pk, pkcs8_from_init) = keystore.init("test", None, seed).unwrap();

        // Load the secret key back and reconstruct PKCS8
        let sk = keystore.secret_key(None).unwrap().unwrap();
        let pkcs8_reconstructed = Keystore::pkcs8_from_secret_key(&sk);

        assert_eq!(pkcs8_from_init, pkcs8_reconstructed);
        assert_eq!(pkcs8_reconstructed.len(), 83);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo test -p radicle-crypto --lib keystore::tests::test_pkcs8_from_secret_key_matches_init`
Expected: FAIL — `pkcs8_from_secret_key` doesn't exist yet.

**Step 3: Write the implementation**

Add this method to the `impl Keystore` block in `keystore.rs`, right after the `init` method (after line 132):

```rust
    /// Reconstruct Ed25519 PKCS8 v2 DER bytes from an existing secret key.
    ///
    /// This produces the same 83-byte encoding as `init()`, suitable for
    /// passing to `auths_id::keri::inception::create_keri_identity_from_key()`.
    pub fn pkcs8_from_secret_key(sk: &SecretKey) -> Vec<u8> {
        let seed_bytes: &[u8] = &sk[..32];
        let pk_bytes: &[u8] = &sk[32..];

        let mut pkcs8 = Vec::with_capacity(83);
        // SEQUENCE (81 bytes total payload)
        pkcs8.extend_from_slice(&[0x30, 0x51]);
        //   INTEGER 1 (version = v2)
        pkcs8.extend_from_slice(&[0x02, 0x01, 0x01]);
        //   SEQUENCE (5 bytes) containing OID
        pkcs8.extend_from_slice(&[0x30, 0x05]);
        //     OID 1.3.101.112 (id-EdDSA / Ed25519)
        pkcs8.extend_from_slice(&[0x06, 0x03, 0x2b, 0x65, 0x70]);
        //   OCTET STRING (34 bytes) wrapping inner OCTET STRING
        pkcs8.extend_from_slice(&[0x04, 0x22]);
        //     OCTET STRING (32 bytes) containing the seed
        pkcs8.extend_from_slice(&[0x04, 0x20]);
        pkcs8.extend_from_slice(seed_bytes);
        //   [1] IMPLICIT (33 bytes) containing the public key
        pkcs8.extend_from_slice(&[0x81, 0x21, 0x00]);
        pkcs8.extend_from_slice(pk_bytes);

        pkcs8
    }
```

**Step 4: Run test to verify it passes**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo test -p radicle-crypto --lib keystore::tests::test_pkcs8_from_secret_key_matches_init`
Expected: PASS

**Step 5: Commit**

```bash
git add *
git commit -m "feat(crypto): add Keystore::pkcs8_from_secret_key for migration"
```

---

### Task 2: Add `Profile::ensure_keri_identity()` migration helper

This is the core migration function. It checks if `keri-prefix` exists; if not, it loads the secret key, reconstructs PKCS8, runs KERI inception, and writes the files. It's designed to be called from both `load()` and from CLI code that has a passphrase.

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/profile.rs`

**Step 1: Write the failing test**

Add to the existing `mod test` block (after line 885):

```rust
    #[test]
    fn test_ensure_keri_identity_migrates_existing_profile() {
        use crate::crypto::{Seed, ssh::Keystore};

        let tmp = tempfile::tempdir().unwrap();
        let home = Home::new(tmp.path().join("radicle")).unwrap();
        let keystore = Keystore::new(&home.keys());
        let seed = Seed::generate();
        let (_pk, _pkcs8) = keystore.init("radicle", None, seed).unwrap();

        // No keri-prefix should exist yet
        assert!(!home.keri_prefix().exists());

        // Run migration
        Profile::ensure_keri_identity(&home, &keystore, None);

        // keri-prefix should now exist
        assert!(home.keri_prefix().exists());
        let prefix = fs::read_to_string(home.keri_prefix()).unwrap();
        assert!(prefix.starts_with("E"), "KERI prefix should start with 'E', got: {}", prefix);

        // keri-next should exist
        assert!(home.keys().join("keri-next").exists());

        // keri git repo should exist
        assert!(home.keys().join("keri").exists());
    }

    #[test]
    fn test_ensure_keri_identity_is_idempotent() {
        use crate::crypto::{Seed, ssh::Keystore};

        let tmp = tempfile::tempdir().unwrap();
        let home = Home::new(tmp.path().join("radicle")).unwrap();
        let keystore = Keystore::new(&home.keys());
        let seed = Seed::generate();
        let (_pk, _pkcs8) = keystore.init("radicle", None, seed).unwrap();

        // First migration
        Profile::ensure_keri_identity(&home, &keystore, None);
        let prefix1 = fs::read_to_string(home.keri_prefix()).unwrap();

        // Second call should be a no-op
        Profile::ensure_keri_identity(&home, &keystore, None);
        let prefix2 = fs::read_to_string(home.keri_prefix()).unwrap();

        assert_eq!(prefix1, prefix2, "Migration must be idempotent");
    }
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo test -p radicle --lib profile::test::test_ensure_keri_identity_migrates_existing_profile`
Expected: FAIL — `ensure_keri_identity` doesn't exist.

**Step 3: Write the implementation**

Add this method to `impl Profile` in `profile.rs`, right after the `keri_prefix()` method (after line 353):

```rust
    /// Ensure a KERI identity exists for this profile.
    ///
    /// If `keri-prefix` already exists, this is a no-op.
    /// Otherwise, loads the secret key (using passphrase if needed),
    /// reconstructs PKCS8 bytes, runs KERI inception, and writes
    /// `keri-prefix` and `keri-next` files.
    ///
    /// This is best-effort: if the key is encrypted and no passphrase
    /// is available, or if any other error occurs, it logs a warning
    /// and returns silently. The profile continues working as did:key.
    pub fn ensure_keri_identity(
        home: &Home,
        keystore: &Keystore,
        passphrase: Option<Passphrase>,
    ) {
        // Already migrated — nothing to do.
        if home.keri_prefix().exists() {
            return;
        }

        // Try to load the secret key.
        let sk = match keystore.secret_key(passphrase) {
            Ok(Some(sk)) => sk,
            Ok(None) => return, // No key on disk
            Err(e) => {
                log::debug!(target: "radicle", "KERI migration skipped: {e}");
                return;
            }
        };

        // Reconstruct PKCS8 v2 DER from existing key.
        let pkcs8_bytes = Keystore::pkcs8_from_secret_key(&sk);

        // Initialize the KERI KEL git repo.
        let keri_repo_path = home.keys().join("keri");
        let keri_repo = match git2::Repository::init(&keri_repo_path) {
            Ok(r) => r,
            Err(e) => {
                log::warn!(target: "radicle", "KERI migration: failed to init KEL repo: {e}");
                return;
            }
        };

        // Run inception.
        let inception = match auths_id::keri::inception::create_keri_identity_from_key(
            &keri_repo,
            &pkcs8_bytes,
            None,
            chrono::Utc::now(),
        ) {
            Ok(r) => r,
            Err(e) => {
                log::warn!(target: "radicle", "KERI migration: inception failed: {e}");
                return;
            }
        };

        // Write prefix file.
        if let Err(e) = std::fs::write(home.keri_prefix(), inception.prefix.as_str()) {
            log::warn!(target: "radicle", "KERI migration: failed to write prefix: {e}");
            return;
        }

        // Write next-rotation key.
        let next_key_path = home.keys().join("keri-next");
        if let Err(e) = std::fs::write(&next_key_path, inception.next_keypair_pkcs8.as_ref()) {
            log::warn!(target: "radicle", "KERI migration: failed to write next key: {e}");
            return;
        }

        log::info!(target: "radicle", "Identity upgraded to did:keri:{}", inception.prefix.as_str());
    }
```

**Step 4: Run tests to verify they pass**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo test -p radicle --lib profile::test::test_ensure_keri_identity`
Expected: PASS (both tests)

**Step 5: Commit**

```bash
git add *
git commit -m "feat(profile): add ensure_keri_identity migration helper"
```

---

### Task 3: Call `ensure_keri_identity` from `Profile::load()`

Wire the migration into the profile load path so it runs automatically.

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/profile.rs:298-321`

**Step 1: Write the failing test**

```rust
    #[test]
    fn test_load_triggers_migration() {
        use crate::crypto::{Seed, ssh::Keystore};

        let tmp = tempfile::tempdir().unwrap();
        let home_path = tmp.path().join("radicle");

        // Simulate an old profile: keys exist but no keri-prefix
        let home = Home::new(&home_path).unwrap();
        let keystore = Keystore::new(&home.keys());
        let seed = Seed::generate();
        let (pk, _) = keystore.init("radicle", None, seed).unwrap();

        // Create minimal config so Profile::load works
        Config::init("test-alias".to_string().try_into().unwrap(), home.config().as_path()).unwrap();

        // No keri-prefix yet
        assert!(!home.keri_prefix().exists());

        // Set RAD_HOME to our temp dir and load
        std::env::set_var("RAD_HOME", &home_path);
        let profile = Profile::load().unwrap();
        std::env::remove_var("RAD_HOME");

        // After load, keri-prefix should exist
        assert!(home.keri_prefix().exists());
    }
```

Note: This test may need `serial_test` or careful env var handling. If parallel test execution causes issues, mark it `#[serial]` or use a unique env setup. The important thing is the assertion that `load()` creates the `keri-prefix` file.

**Step 2: Modify `Profile::load()`**

In `profile.rs`, change `Profile::load()` (starting at line 298) to call migration after loading keys but before returning:

```rust
    pub fn load() -> Result<Self, Error> {
        let home = self::home()?;
        let keystore = Keystore::new(&home.keys());
        let public_key = keystore
            .public_key()?
            .ok_or_else(|| Error::NotFound(home.path().to_path_buf()))?;

        // Auto-migrate did:key → did:keri (best-effort, silent on failure).
        Self::ensure_keri_identity(&home, &keystore, env::passphrase());

        let config = Config::load(home.config().as_path())?;
        let storage = Storage::open(
            home.storage(),
            git::UserInfo {
                alias: config.alias().clone(),
                key: public_key,
            },
        )?;
        transport::local::register(storage.clone());

        Ok(Profile {
            home,
            storage,
            keystore,
            public_key,
            config,
        })
    }
```

The only change is adding the `Self::ensure_keri_identity(...)` line after `public_key` is loaded.

**Step 3: Run test to verify it passes**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo test -p radicle --lib profile::test::test_load_triggers_migration`
Expected: PASS

**Step 4: Commit**

```bash
git add *
git commit -m "feat(profile): auto-migrate did:key to did:keri on load"
```

---

### Task 4: DRY up `Profile::init()` to use `ensure_keri_identity`

Now that migration is a standalone function, `Profile::init()` should use it too instead of duplicating the KERI inception logic.

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/profile.rs:232-296`

**Step 1: Refactor `Profile::init()`**

Replace lines 241-259 (the inline KERI inception block) with a single call:

```rust
    pub fn init(
        home: Home,
        alias: Alias,
        passphrase: Option<Passphrase>,
        seed: crypto::Seed,
    ) -> Result<Self, Error> {
        let keystore = Keystore::new(&home.keys());
        let (public_key, _pkcs8_bytes) = keystore.init("radicle", passphrase.clone(), seed)?;

        // Create KERI identity using the radicle key as the first device.
        Self::ensure_keri_identity(&home, &keystore, passphrase);

        let config = Config::init(alias.clone(), home.config().as_path())?;
        // ... rest unchanged
```

The `_pkcs8_bytes` is now unused from init's return — `ensure_keri_identity` reconstructs them internally. This is intentional: single code path.

**Step 2: Run existing tests**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo test -p radicle`
Expected: All tests PASS (no behavior change, just DRY)

**Step 3: Commit**

```bash
git add *
git commit -m "refactor(profile): DRY init() to use ensure_keri_identity"
```

---

### Task 5: Add migration notice to `rad auth` for encrypted keys

For users with encrypted keys who don't have `RAD_PASSPHRASE` set, `Profile::load()` will skip migration (can't decrypt). We catch this case in the `rad auth` CLI flow where the user provides their passphrase interactively.

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle-cli/src/commands/auth.rs`

**Step 1: Find the existing auth flow**

Look for where the user provides a passphrase during `rad auth` (the login/unlock flow, not the init flow). After the passphrase is validated and the key is loaded into ssh-agent, call migration.

Add after the key is unlocked:

```rust
// After successful auth with passphrase, ensure KERI migration
Profile::ensure_keri_identity(
    &profile.home,
    &profile.keystore,
    Some(passphrase.clone()),
);
```

**Step 2: Run `rad auth` manually against an old profile**

Run: Create a test profile without KERI, then run `rad auth`. Verify `keri-prefix` appears.

**Step 3: Commit**

```bash
git add *
git commit -m "feat(cli): trigger KERI migration on rad auth for encrypted keys"
```

---

### Task 6: Integration test — full migration round-trip

Verify the complete flow: create old-style profile → load → verify did:keri.

**Files:**
- Modify: `/Users/bordumb/workspace/repositories/radicle-base/heartwood/crates/radicle/src/profile.rs` (test module)

**Step 1: Write integration test**

```rust
    #[test]
    fn test_migration_produces_valid_did_keri() {
        use crate::crypto::{Seed, ssh::Keystore};

        let tmp = tempfile::tempdir().unwrap();
        let home = Home::new(tmp.path().join("radicle")).unwrap();
        let keystore = Keystore::new(&home.keys());
        let seed = Seed::generate();
        let (pk, _) = keystore.init("radicle", None, seed).unwrap();

        // Before migration: did() should return did:key
        // (We can't call profile.did() without a full Profile, so check file state)
        assert!(!home.keri_prefix().exists());

        // Migrate
        Profile::ensure_keri_identity(&home, &keystore, None);

        // After migration: keri-prefix should exist and be non-empty
        let prefix = fs::read_to_string(home.keri_prefix()).unwrap();
        assert!(!prefix.is_empty());
        assert!(prefix.starts_with('E'), "KERI prefix format: {prefix}");

        // Verify idempotency
        Profile::ensure_keri_identity(&home, &keystore, None);
        let prefix2 = fs::read_to_string(home.keri_prefix()).unwrap();
        assert_eq!(prefix, prefix2);

        // Verify the keri KEL repo has at least one commit
        let keri_repo = git2::Repository::open(home.keys().join("keri")).unwrap();
        let head = keri_repo.head().unwrap();
        assert!(head.peel_to_commit().is_ok(), "KEL repo should have inception commit");
    }
```

**Step 2: Run test**

Run: `cd /Users/bordumb/workspace/repositories/radicle-base/heartwood && cargo test -p radicle --lib profile::test::test_migration_produces_valid_did_keri`
Expected: PASS

**Step 3: Commit**

```bash
git add *
git commit -m "test(profile): add integration test for did:key to did:keri migration"
```

---

### Summary

| Task | What | Files |
|------|------|-------|
| 1 | `Keystore::pkcs8_from_secret_key()` | `radicle-crypto/src/ssh/keystore.rs` |
| 2 | `Profile::ensure_keri_identity()` | `radicle/src/profile.rs` |
| 3 | Wire into `Profile::load()` | `radicle/src/profile.rs` |
| 4 | DRY up `Profile::init()` | `radicle/src/profile.rs` |
| 5 | Wire into `rad auth` for encrypted keys | `radicle-cli/src/commands/auth.rs` |
| 6 | Integration test | `radicle/src/profile.rs` |

**Migration behavior:**
- Unencrypted keys: migrated silently on first `rad` command (any command that calls `Profile::load()`)
- Encrypted keys with `RAD_PASSPHRASE` env: migrated silently on first `rad` command
- Encrypted keys without env passphrase: migrated on next `rad auth` (when user provides passphrase)
- Already migrated: no-op (idempotent)
- Migration failure: silent, profile continues as `did:key`, logged at `warn` level
