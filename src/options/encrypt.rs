use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// Input dir to scan images
    #[arg(long, short = 'i', value_name = "DIR")]
    pub input: PathBuf,

    /// Output dir to write encrypted files
    #[arg(long, short = 'o', value_name = "DIR")]
    pub output: PathBuf,

    /// Hex key file (64 hex chars = 32 bytes AES-256-GCM)
    #[arg(long, short = 'k', value_name = "FILE")]
    pub key_file: PathBuf,

    /// Overwrite existing .lock files if present
    #[arg(long)]
    pub overwrite: bool,

    /// Include hidden files (dotfiles)
    #[arg(long)]
    pub include_hidden: bool,

    /// Prompt for passphrase if the key file is protected
    #[arg(long = "passphrase-prompt", short = 'p', visible_alias = "pp")]
    pub passphrase_prompt: bool,
}
