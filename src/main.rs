mod cli;
mod commands;
mod options;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Keygen(args) => commands::keygen::run(args),
    }
}
