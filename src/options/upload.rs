use clap::{Args, ValueEnum};
use std::path::PathBuf;
use lock::colors;

#[derive(ValueEnum, Clone, Debug)]
pub enum Provider {
    /// Upload to GitHub repository
    Github,
    /// Upload to Google Drive (not implemented)
    Drive,
    /// Upload to Microsoft OneDrive (not implemented)
    Onedrive,
}

#[derive(Args, Debug)]
pub struct UploadArgs {
    /// Cloud storage provider
    #[arg(value_enum, help_heading = colors::PROVIDER_OPTIONS_HEADING)]
    pub provider: Provider,

    // Path options
    /// Directory containing encrypted files to upload
    #[arg(long, short = 'i', value_name = "DIR", help_heading = colors::PATH_OPTIONS_HEADING)]
    pub input_dir: PathBuf,

    // GitHub options
    /// GitHub personal access token [env: GITHUB_TOKEN]
    #[arg(
        long = "github-token",
        short = 't',
        env = "GITHUB_TOKEN",
        value_name = "TOKEN",
        help_heading = colors::GITHUB_OPTIONS_HEADING
    )]
    pub github_token: Option<String>,

    /// Target GitHub repository (format: owner/repo)
    #[arg(long = "github-repo", value_name = "OWNER/REPO", help_heading = colors::GITHUB_OPTIONS_HEADING)]
    pub github_repo: Option<String>,

    /// Target branch for upload [default: main]
    #[arg(
        long = "github-branch",
        short = 'b',
        value_name = "BRANCH",
        default_value = "main",
        help_heading = colors::GITHUB_OPTIONS_HEADING
    )]
    pub github_branch: String,

    // Output options
    /// Show detailed progress for each file
    #[arg(long, short = 'v', help_heading = colors::OUTPUT_OPTIONS_HEADING)]
    pub verbose: bool,
}
