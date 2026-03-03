use crate::config::Config;
use crate::git;
use anyhow::{bail, Result};
use chrono::Local;
use colored::Colorize;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

pub fn run(config: &Config, dir: Option<&str>, debounce: Option<u64>) -> Result<()> {
    let watch_dir = match dir {
        Some(d) => PathBuf::from(d),
        None => env::current_dir()?,
    };

    if !watch_dir.join(".git").exists() {
        bail!("Not a git repository: {}", watch_dir.display());
    }

    let debounce_secs = debounce.unwrap_or(config.watch_debounce_seconds);
    let debounce_duration = Duration::from_secs(debounce_secs);

    println!(
        "{} Watching {} (debounce: {}s)",
        "→".cyan(),
        watch_dir.display(),
        debounce_secs
    );
    println!("  Press Ctrl-C to stop.");

    let (tx, rx) = mpsc::channel();
    let mut debouncer = new_debouncer(debounce_duration, tx)?;

    debouncer
        .watcher()
        .watch(&watch_dir, RecursiveMode::Recursive)?;

    let git_dir = watch_dir.join(".git");

    for events in rx {
        match events {
            Ok(events) => {
                // Ignore events originating inside .git/
                let relevant = events
                    .iter()
                    .any(|e| !e.path.starts_with(&git_dir));

                if !relevant {
                    continue;
                }

                let repo = git::open_repo(&watch_dir)?;
                if git::has_changes(&repo)? {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M").to_string();
                    let msg = format!("auto: {}", timestamp);
                    git::stage_and_commit(&repo, &msg)?;
                    println!("{} {}", "Auto-committed:".green(), msg);
                }
            }
            Err(e) => {
                eprintln!("{} Watch error: {:?}", "!".red(), e);
            }
        }
    }

    Ok(())
}
