use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug)]
pub enum Algo {
    /// AES-256-GCM (32-byte key)
    #[value(alias = "aes")] // allows: -a aes
    AesGcm,
}

#[derive(Args, Debug)]
pub struct KeygenArgs {
    /// Output file to write the key
    #[arg(long, short, value_name = "FILE", default_value = "lock.key")]
    pub out: PathBuf,

    /// Algorithm for the key (currently: AES-256-GCM)
    #[arg(long, short, value_enum, default_value_t = Algo::AesGcm)]
    pub algo: Algo,

    /// Overwrite output file if it exists
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Prompt for passphrase to protect the key file (Argon2id + AES-256-GCM)
    #[arg(long = "passphrase-prompt", short = 'p', visible_alias = "pp")]
    pub passphrase_prompt: bool,
}
