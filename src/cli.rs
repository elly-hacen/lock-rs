use clap::{Parser, Subcommand};

use crate::options;

#[derive(Parser)]
#[command(
    name = "lock",
    version,
    about = "Yet another paranoid lock - zeroidize your personal pixels before they hit the cloud."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new encryption key
    #[command(
        visible_alias = "kgen",
        long_about = "Generate a new encryption key file.\n\nCreates a 256-bit AES key for encrypting files. The key can optionally be protected with a passphrase."
    )]
    Keygen(options::keygen::KeygenArgs),

    /// Encrypt files with AES-256-GCM
    #[command(
        visible_alias = "enc",
        long_about = "Encrypt files with AES-256-GCM encryption.\n\nRecursively scans directories for supported file types and encrypts them."
    )]
    Encrypt(options::encrypt::EncryptArgs),

    /// Decrypt encrypted files
    #[command(
        visible_alias = "dec",
        long_about = "Decrypt encrypted files.\n\nDecrypts all .lock files in a directory back to their original format."
    )]
    Decrypt(options::decrypt::DecryptArgs),

    /// Inspect encrypted files or keys
    #[command(
        visible_alias = "insp",
        long_about = "Inspect encrypted files or keys.\n\nDisplays metadata and information about encrypted files or key files."
    )]
    Inspect(options::inspect::InspectArgs),

    /// Upload encrypted files to GitHub
    #[command(
        visible_alias = "up",
        long_about = "Upload encrypted files to GitHub.\n\nUploads encrypted files to a GitHub repository for secure cloud storage."
    )]
    Upload(options::upload::UploadArgs),
}
