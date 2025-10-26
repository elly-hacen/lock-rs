use clap::{Args, ValueEnum};
use std::path::PathBuf;
use lock::colors;

#[derive(ValueEnum, Clone, Debug)]
pub enum Algo {
    /// AES-256-GCM (32-byte key)
    #[value(alias = "aes")] // allows: -a aes
    AesGcm,
}

#[derive(Args, Debug)]
pub struct KeygenArgs {
    // Key options
    /// Output file for the encryption key [default: lock.key]
    #[arg(long, short, value_name = "FILE", default_value = "lock.key", help_heading = colors::KEY_OPTIONS_HEADING)]
    pub out: PathBuf,

    /// Encryption algorithm [default: aes-gcm]
    #[arg(long, short, value_enum, default_value_t = Algo::AesGcm, help_heading = colors::KEY_OPTIONS_HEADING)]
    pub algo: Algo,

    // Security options
    /// Prompt for passphrase to protect the key
    #[arg(long = "passphrase-prompt", short = 'p', visible_alias = "pp", help_heading = colors::SECURITY_OPTIONS_HEADING)]
    pub passphrase_prompt: bool,

    // File options
    /// Overwrite existing key file
    #[arg(long, short = 'f', help_heading = colors::FILE_OPTIONS_HEADING)]
    pub force: bool,
}
