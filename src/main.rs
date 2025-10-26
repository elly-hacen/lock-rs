mod cli;
mod commands;
mod options;

use clap::Parser;
use cli::{Cli, Commands};
use std::env;
use lock::print_help;

fn print_custom_help() {
    print_help!(header: "Yet another paranoid lock - zeroidize your personal pixels before they hit the cloud.");
    print_help!(usage: "lock <COMMAND>");
    
    print_help!(section: "Commands:");
    print_help!(command: "keygen ", "Generate a new encryption key [aliases: kgen]");
    print_help!(command: "encrypt", "Encrypt files with AES-256-GCM [aliases: enc]");
    print_help!(command: "decrypt", "Decrypt encrypted files [aliases: dec]");
    print_help!(command: "inspect", "Inspect encrypted files or keys [aliases: insp]");
    print_help!(command: "upload ", "Upload encrypted files to GitHub [aliases: up]");
    print_help!(command: "help   ", "Print this message or the help of the given subcommand(s)");
    println!();
    
    print_help!(section: "Options:");
    print_help!(option: "-h, --help    ", "Print help");
    print_help!(option: "-V, --version ", "Print version");
    println!();
    
    print_help!(footer: "Use `lock help <command>` for more details on a specific command.");
}

fn main() -> anyhow::Result<()> {
    // Check if help is requested
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 || (args.len() == 2 && (args[1] == "--help" || args[1] == "-h" || args[1] == "help")) {
        print_custom_help();
        return Ok(());
    }
    
    let cli = Cli::parse();
    match cli.command {
        Commands::Keygen(args) => commands::keygen::run(args),
        Commands::Encrypt(args) => commands::encrypt::run(args),
        Commands::Decrypt(args) => commands::decrypt::run(args),
        Commands::Inspect(args) => commands::inspect::run(args),
        Commands::Upload(args) => commands::upload::run(args),
    }
}
