use clap::Args;
use std::path::PathBuf;
use lock::colors;

#[derive(Args, Debug)]
pub struct DecryptArgs {
    // Path options
    /// Directory containing encrypted .lock files
    #[arg(long, short = 'i', value_name = "DIR", help_heading = colors::PATH_OPTIONS_HEADING)]
    pub input: PathBuf,

    /// Directory for decrypted output files
    #[arg(long, short = 'o', value_name = "DIR", help_heading = colors::PATH_OPTIONS_HEADING)]
    pub output: PathBuf,

    /// Encryption key file (256-bit AES key)
    #[arg(long, short = 'k', value_name = "FILE", help_heading = colors::PATH_OPTIONS_HEADING)]
    pub key_file: PathBuf,

    // Decryption options
    /// Include hidden files and directories
    #[arg(long, help_heading = colors::DECRYPTION_OPTIONS_HEADING)]
    pub include_hidden: bool,

    /// Prompt for passphrase if key is protected
    #[arg(long = "passphrase-prompt", short = 'p', visible_alias = "pp", help_heading = colors::DECRYPTION_OPTIONS_HEADING)]
    pub passphrase_prompt: bool,

    // Performance options
    /// Number of parallel workers [default: number of CPUs]
    #[arg(long = "jobs", short = 'j', value_name = "N", help_heading = colors::PERFORMANCE_OPTIONS_HEADING)]
    pub jobs: Option<usize>,

    // Output options
    /// Show detailed progress for each file
    #[arg(long, short = 'v', help_heading = colors::OUTPUT_OPTIONS_HEADING)]
    pub verbose: bool,
}
