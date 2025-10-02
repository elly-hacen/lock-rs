use clap::{Args, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Debug)]
pub enum Algo {
    /// AES-256 (32-byte key)
    Aes,
}

#[derive(Args, Debug)]
pub struct KeygenArgs {
    /// Output file to write the key
    #[arg(long, short, value_name = "FILE", default_value = "lock.key")]
    pub out: PathBuf,

    /// Algorithm for the key (currently: AES-256)
    #[arg(long, short, value_enum, default_value_t = Algo::Aes)]
    pub algo: Algo,

    /// Overwrite output file if it exists
    #[arg(long, short = 'f')]
    pub force: bool,
}
