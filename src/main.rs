mod cli;
mod engine;
mod models;
mod scanner;
mod storage;

use clap::Parser;
use cli::{Cli, Commands};
use std::error::Error;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { source, backup } => {
            engine::init(&source, &backup)?;
        }

        Commands::Status { backup } => {
            engine::status(&backup)?;
        }

        Commands::Backup { backup } => {
            engine::backup(&backup)?;
        }

        Commands::History { backup } => {
            engine::history(&backup)?;
        }

        Commands::Restore {
            backup,
            snapshot,
            overwrite,
        } => {
            engine::restore(&backup, &snapshot, overwrite)?;
        }

        Commands::Verify { backup, snapshot } => {
            engine::verify(&backup, &snapshot)?;
        }
    }

    Ok(())
}
