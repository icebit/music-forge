use crate::commands::snapshot;
use crate::config::Config;
use crate::git;
use anyhow::bail;
use anyhow::Result;
use colored::Colorize;
use std::env;

pub fn run(config: &Config, message: &str, do_snapshot: bool) -> Result<()> {
    let cwd = env::current_dir()?;
    let repo = git::open_repo(&cwd)?;

    if do_snapshot {
        snapshot::render_if_available(config, &cwd)?;
    }

    if !git::has_changes(&repo)? {
        bail!("Nothing to commit (working tree clean)");
    }

    let commit_msg = if do_snapshot {
        format!("snapshot: {}", message)
    } else {
        message.to_string()
    };

    git::stage_and_commit(&repo, &commit_msg)?;
    println!("{} {}", "Committed:".green(), commit_msg);

    Ok(())
}
