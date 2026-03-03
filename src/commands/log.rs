use crate::git;
use anyhow::bail;
use anyhow::Result;
use colored::Colorize;
use std::env;

pub fn run(message: &str) -> Result<()> {
    let cwd = env::current_dir()?;
    let repo = git::open_repo(&cwd)?;

    if !git::has_changes(&repo)? {
        bail!("Nothing to commit (working tree clean)");
    }

    git::stage_and_commit(&repo, message)?;
    println!("{} {}", "Committed:".green(), message);

    Ok(())
}
