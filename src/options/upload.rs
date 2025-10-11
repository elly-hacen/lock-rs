use clap::{Args, ValueEnum};
use std::path::PathBuf;

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
    #[arg(value_enum)]
    pub provider: Provider,

    /// Directory containing encrypted files (.lock)
    #[arg(long, short = 'i', value_name = "DIR")]
    pub input_dir: PathBuf,

    /// GitHub personal access token (PAT Token).
    /// If not provided, will fall back to env var GITHUB_TOKEN.
    #[arg(
        long = "github-token",
        short = 't',
        env = "GITHUB_TOKEN",
        value_name = "TOKEN"
    )]
    pub github_token: Option<String>,

    /// GitHub repository name in OWNER/REPO form
    #[arg(long = "github-repo", value_name = "OWNER/REPO")]
    pub github_repo: Option<String>,

    /// GitHub branch to upload to (default: main)
    #[arg(
        long = "github-branch",
        short = 'b',
        value_name = "BRANCH",
        default_value = "main"
    )]
    pub github_branch: String,

    /// Show detailed progress for each file upload
    #[arg(long, short = 'v')]
    pub verbose: bool,
}
