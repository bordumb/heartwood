use clap::Parser;

const ABOUT: &str = "Migrate identity from did:key to did:keri (multi-device support)";

#[derive(Debug, Parser)]
#[command(about = ABOUT, disable_version_flag = true)]
pub struct Args {
    /// Read passphrase from stdin
    #[arg(long, default_value_t = false)]
    pub stdin: bool,

    /// Roll back a previous migration, restoring the legacy did:key identity
    #[arg(long, default_value_t = false)]
    pub rollback: bool,
}
