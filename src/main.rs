mod config;
mod git;
mod projects;
mod reaper;
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
    /// Adopt an existing project into music-forge (run from inside the project dir)
    Adopt {
        /// Adopt all projects in the configured projects directory
        #[arg(long)]
        all: bool,
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
        /// Render in Reaper and commit as a snapshot
        #[arg(long)]
        snapshot: bool,
    },
    /// Render in Reaper and commit as a snapshot
    Snapshot {
        /// Project name (partial match OK); defaults to current directory
        project: Option<String>,
        /// Commit message label
        #[arg(short, long)]
        message: Option<String>,
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
    /// Show the git history of the current project as a timeline
    Timeline,
    /// Open a project or seed by name (partial match supported)
    Open {
        name: String,
    },
    /// Show a summary of all projects
    Dashboard,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load()?;

    match cli.command {
        Commands::Init { name } => commands::init::run(&config, &name),
        Commands::Adopt { all } => {
            if all {
                commands::adopt::run_all(&config)
            } else {
                commands::adopt::run(&config)
            }
        }
        Commands::Seed { name } => commands::seed::run(&config, &name),
        Commands::Promote { seed_path } => commands::promote::run(&config, &seed_path),
        Commands::Log { message, snapshot } => commands::log::run(&config, &message, snapshot),
        Commands::Snapshot { project, message } => commands::snapshot::run(&config, project.as_deref(), message.as_deref()),
        Commands::Watch { dir, debounce } => commands::watch::run(&config, dir.as_deref(), debounce),
        Commands::Status { status } => commands::status::run(&status),
        Commands::Ingest { files, to } => commands::ingest::run(&files, to.as_deref()),
        Commands::Open { name } => commands::open::run(&config, &name),
        Commands::Timeline => commands::timeline::run(),
        Commands::Dashboard => commands::dashboard::run(&config),
    }
}
