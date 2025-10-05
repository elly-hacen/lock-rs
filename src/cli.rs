use clap::{Parser, Subcommand};

use crate::options;

#[derive(Parser)]
#[command(name = "lock", version, about = "Encrypt/Decrypt images with Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new key file
    Keygen(options::keygen::KeygenArgs),

    /// Encrypt all images in a dir using (default: AES-256-GCM)
    Encrypt(options::encrypt::EncryptArgs),

    /// Decrypt all encrypted images in a dir
    Decrypt(options::decrypt::DecryptArgs),
}
