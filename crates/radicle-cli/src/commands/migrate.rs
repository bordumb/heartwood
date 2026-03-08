mod args;

use std::path::Path;

use radicle::profile::env;
use radicle::Profile;

use crate::terminal as term;

pub use args::Args;

pub fn run(args: Args, ctx: impl term::Context) -> anyhow::Result<()> {
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
