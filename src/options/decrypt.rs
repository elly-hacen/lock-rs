use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DecryptArgs {
    /// Input dir to scan encrypted images
    #[arg(long, short = 'i', value_name = "DIR")]
    pub input: PathBuf,

    /// Output dir to write decrypted images
    #[arg(long, short = 'o', value_name = "DIR")]
    pub output: PathBuf,

    /// Hex key file (64 hex chars = 32 bytes AES-256-GCM)
    #[arg(long, short = 'k', value_name = "FILE")]
    pub key_file: PathBuf,

    /// Prompt for passphrase if the key file is protected
    #[arg(long = "passphrase-prompt", short = 'p', visible_alias = "pp")]
    pub passphrase_prompt: bool,

    /// Max parallel workers (default: number of CPUs)
    #[arg(long = "jobs", short = 'j', value_name = "N")]
    pub jobs: Option<usize>,

    /// Verbose output: print one line per file and disable progress bar
    #[arg(long, short = 'v')]
    pub verbose: bool,
}
