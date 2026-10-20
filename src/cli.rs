use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "chrono",
    version,
    about = "A snapshot-based file backup utility"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        #[arg(long)]
        source: PathBuf,

        #[arg(long)]
        backup: PathBuf,
    },

    Status {
        #[arg(long)]
        backup: PathBuf,
    },

    Backup {
        #[arg(long)]
        backup: PathBuf,
    },
}
