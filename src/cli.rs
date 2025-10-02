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
}
