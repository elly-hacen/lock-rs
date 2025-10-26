use clap::Args;
use std::path::PathBuf;
use lock::colors;

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// File to inspect (.lock or .key)
    #[arg(value_name = "FILE", help_heading = colors::INPUT_HEADING)]
    pub file: PathBuf,

    // Output options
    /// Output inspection result as JSON
    #[arg(long, help_heading = colors::OUTPUT_OPTIONS_HEADING)]
    pub json: bool,
}
