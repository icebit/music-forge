mod config;
mod git;
mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "music-forge", about = "Music project management CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new song project
    Init {
        name: String,
    },
    /// Create a new seed (quick idea)
    Seed {
        name: String,
    },
    /// Promote a seed to a full project
    Promote {
        seed_path: String,
    },
    /// Commit current changes with a message
    Log {
        message: String,
    },
    /// Watch a directory and auto-commit changes
    Watch {
        dir: Option<String>,
        #[arg(long)]
        debounce: Option<u64>,
    },
    /// Set the status of the current project
    Status {
        status: String,
    },
    /// Ingest files into the current project
    Ingest {
        files: Vec<String>,
        #[arg(long)]
        to: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;

    match cli.command {
        Commands::Init { name } => commands::init::run(&config, &name),
        Commands::Seed { name } => commands::seed::run(&config, &name),
        Commands::Promote { seed_path } => commands::promote::run(&config, &seed_path),
        Commands::Log { message } => commands::log::run(&message),
        Commands::Watch { dir, debounce } => commands::watch::run(&config, dir.as_deref(), debounce),
        Commands::Status { status } => commands::status::run(&status),
        Commands::Ingest { files, to } => commands::ingest::run(&files, to.as_deref()),
    }
}
