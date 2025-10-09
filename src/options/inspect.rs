use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InspectArgs {
    /// File to inspect (.lock or .key)
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Output JSON text
    #[arg(long)]
    pub json: bool,
}
