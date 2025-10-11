mod cli;
mod commands;
mod options;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Keygen(args) => commands::keygen::run(args),
        Commands::Encrypt(args) => commands::encrypt::run(args),
        Commands::Decrypt(args) => commands::decrypt::run(args),
        Commands::Inspect(args) => commands::inspect::run(args),
        Commands::Upload(args) => commands::upload::run(args),
    }
}
