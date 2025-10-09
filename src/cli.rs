use clap::{Parser, Subcommand};

use crate::options;

#[derive(Parser)]
#[command(
    name = "lock",
    version,
    about = "Yet another paranoid lock - zerodize your personal pixels before they hit the cloud."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new key file
    #[command(visible_alias = "kgen")]
    Keygen(options::keygen::KeygenArgs),

    /// Encrypt all images in a dir using (default: AES-256-GCM)
    #[command(visible_alias = "enc")]
    Encrypt(options::encrypt::EncryptArgs),

    /// Decrypt all .lock files in a dir using (default: AES-256-GCM)
    #[command(visible_alias = "dec")]
    Decrypt(options::decrypt::DecryptArgs),

    /// File to inspect (.lock or .key)
    #[command(visible_alias = "insp")]
    Inspect(options::inspect::InspectArgs),
}
