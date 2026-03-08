mod args;

use std::path::Path;

use radicle::profile::env;
use radicle::Profile;

use crate::terminal as term;

pub use args::Args;

pub fn run(args: Args, ctx: impl term::Context) -> anyhow::Result<()> {
    if args.rollback {
        return rollback(ctx);
    }
    migrate(args, ctx)
}

fn migrate(args: Args, ctx: impl term::Context) -> anyhow::Result<()> {
    let profile = ctx.profile()?;

    // Already migrated
    if profile.keri_prefix().is_some() {
        term::success!(
            "Already migrated to {}",
            term::format::highlight(profile.did())
        );
        return Ok(());
    }

    // Back up legacy identity files before migration
    let keys_dir = profile.home.keys();
    let backup_dir = profile.home.path().join("keys-backup-pre-keri");
    backup_legacy_keys(&keys_dir, &backup_dir)?;
    term::success!(
        "Legacy identity backed up to {}",
        term::format::dim(backup_dir.display())
    );

    // Need passphrase if key is encrypted
    let passphrase = if !profile.keystore.is_encrypted()? {
        None
    } else if let Some(p) = env::passphrase() {
        Some(p)
    } else if args.stdin {
        Some(term::passphrase_stdin()?)
    } else if let Some(p) = term::io::passphrase(
        term::io::PassphraseValidator::new(profile.keystore.clone()),
    )? {
        Some(p)
    } else {
        anyhow::bail!("A passphrase is required to migrate your identity.");
    };

    let spinner = term::spinner("Migrating identity to did:keri...");
    Profile::ensure_keri_identity(&profile.home, &profile.keystore, passphrase);
    spinner.finish();

    // Verify migration succeeded
    // Re-check keri_prefix by reading from disk directly
    let prefix_path = profile.home.keri_prefix();
    if let Ok(prefix) = std::fs::read_to_string(&prefix_path) {
        let prefix = prefix.trim().to_string();
        term::success!(
            "Identity migrated to {}",
            term::format::highlight(format!("did:keri:{prefix}"))
        );
        term::info!(
            "Your device key {} is now linked to your new identity.",
            term::format::tertiary(profile.id())
        );
    } else {
        anyhow::bail!("Migration failed. Check that your key is accessible and try again.");
    }

    Ok(())
}

fn rollback(ctx: impl term::Context) -> anyhow::Result<()> {
    let profile = ctx.profile()?;

    // Check that there's a migration to roll back
    if profile.keri_prefix().is_none() {
        term::info!("No KERI migration found. Already on legacy did:key identity.");
        return Ok(());
    }

    let keys_dir = profile.home.keys();
    let backup_dir = profile.home.path().join("keys-backup-pre-keri");

    // Verify backup exists
    if !backup_dir.exists() {
        anyhow::bail!(
            "No backup found at {}. Cannot roll back without a backup.",
            backup_dir.display()
        );
    }

    // Restore backed-up key files into keys/
    restore_legacy_keys(&backup_dir, &keys_dir)?;
    term::success!(
        "Restored legacy keys from {}",
        term::format::dim(backup_dir.display())
    );

    // Remove KERI artifacts
    remove_keri_artifacts(&keys_dir)?;
    term::success!("Removed KERI identity artifacts");

    term::success!(
        "Rolled back to legacy identity {}",
        term::format::highlight(format!(
            "did:key:{}",
            profile.id()
        ))
    );
    term::info!(
        "{}",
        term::format::dim("The backup directory has been preserved in case you need it again.")
    );

    Ok(())
}

/// Copy `radicle` and `radicle.pub` into a backup directory.
fn backup_legacy_keys(keys_dir: &Path, backup_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(backup_dir)?;

    for name in &["radicle", "radicle.pub"] {
        let src = keys_dir.join(name);
        if src.exists() {
            std::fs::copy(&src, backup_dir.join(name))?;
        }
    }
    // Restrict backup directory permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(backup_dir, std::fs::Permissions::from_mode(0o700));
    }
    Ok(())
}

/// Copy `radicle` and `radicle.pub` from backup back into keys/.
fn restore_legacy_keys(backup_dir: &Path, keys_dir: &Path) -> anyhow::Result<()> {
    for name in &["radicle", "radicle.pub"] {
        let src = backup_dir.join(name);
        if src.exists() {
            std::fs::copy(&src, keys_dir.join(name))?;
        } else {
            anyhow::bail!(
                "Backup file {} is missing. Cannot complete rollback.",
                src.display()
            );
        }
    }
    Ok(())
}

/// Remove keri-prefix, keri-next, and keri/ directory from keys/.
fn remove_keri_artifacts(keys_dir: &Path) -> anyhow::Result<()> {
    let keri_prefix = keys_dir.join("keri-prefix");
    if keri_prefix.exists() {
        std::fs::remove_file(&keri_prefix)?;
    }

    let keri_next = keys_dir.join("keri-next");
    if keri_next.exists() {
        std::fs::remove_file(&keri_next)?;
    }

    let keri_dir = keys_dir.join("keri");
    if keri_dir.exists() {
        std::fs::remove_dir_all(&keri_dir)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use radicle::crypto::{ssh::Keystore, Seed};
    use radicle::profile::Home;
    use std::fs;

    /// Helper: set up a temp home with a keypair and run KERI migration + backup.
    fn setup_migrated_home() -> (tempfile::TempDir, Home, Keystore) {
        let tmp = tempfile::tempdir().unwrap();
        let home = Home::new(tmp.path().join("radicle")).unwrap();
        let keystore = Keystore::new(&home.keys());
        let seed = Seed::generate();
        let (_pk, _pkcs8) = keystore.init("radicle", None, seed).unwrap();

        // Back up before migration (matches what `rad migrate` does)
        let backup_dir = home.path().join("keys-backup-pre-keri");
        backup_legacy_keys(&home.keys(), &backup_dir).unwrap();

        // Migrate
        Profile::ensure_keri_identity(&home, &keystore, None);
        assert!(home.keri_prefix().exists(), "migration should create keri-prefix");

        (tmp, home, keystore)
    }

    #[test]
    fn test_rollback_restores_legacy_keys() {
        let (_tmp, home, _keystore) = setup_migrated_home();

        // Record the backed-up key contents
        let backup_dir = home.path().join("keys-backup-pre-keri");
        let backup_priv = fs::read(backup_dir.join("radicle")).unwrap();
        let backup_pub = fs::read(backup_dir.join("radicle.pub")).unwrap();

        // Perform rollback
        let keys_dir = home.keys();
        restore_legacy_keys(&backup_dir, &keys_dir).unwrap();
        remove_keri_artifacts(&keys_dir).unwrap();

        // Keys should match the backup
        let restored_priv = fs::read(keys_dir.join("radicle")).unwrap();
        let restored_pub = fs::read(keys_dir.join("radicle.pub")).unwrap();
        assert_eq!(backup_priv, restored_priv, "private key should match backup");
        assert_eq!(backup_pub, restored_pub, "public key should match backup");

        // KERI artifacts should be gone
        assert!(!home.keri_prefix().exists(), "keri-prefix should be removed");
        assert!(!keys_dir.join("keri-next").exists(), "keri-next should be removed");
        assert!(!keys_dir.join("keri").exists(), "keri/ dir should be removed");
    }

    #[test]
    fn test_rollback_then_remigrate() {
        let (_tmp, home, keystore) = setup_migrated_home();

        let backup_dir = home.path().join("keys-backup-pre-keri");
        let keys_dir = home.keys();

        // Roll back
        restore_legacy_keys(&backup_dir, &keys_dir).unwrap();
        remove_keri_artifacts(&keys_dir).unwrap();
        assert!(!home.keri_prefix().exists());

        // Re-migrate should succeed (prefix may differ due to random next-rotation key)
        Profile::ensure_keri_identity(&home, &keystore, None);
        assert!(home.keri_prefix().exists(), "re-migration should create keri-prefix");
        let new_prefix = fs::read_to_string(home.keri_prefix()).unwrap();
        assert!(
            new_prefix.starts_with('E'),
            "re-migrated prefix should be valid KERI format, got: {new_prefix}"
        );
    }

    #[test]
    fn test_rollback_fails_without_backup() {
        let tmp = tempfile::tempdir().unwrap();
        let home = Home::new(tmp.path().join("radicle")).unwrap();
        let keystore = Keystore::new(&home.keys());
        let seed = Seed::generate();
        let (_pk, _pkcs8) = keystore.init("radicle", None, seed).unwrap();

        // Migrate WITHOUT creating a backup first
        Profile::ensure_keri_identity(&home, &keystore, None);
        assert!(home.keri_prefix().exists());

        // Rollback should fail because no backup dir exists
        let backup_dir = home.path().join("keys-backup-pre-keri");
        assert!(!backup_dir.exists(), "no backup should exist");
    }
}
