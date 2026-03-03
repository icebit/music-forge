use crate::config::Config;
use crate::git;
use crate::projects;
use crate::reaper;
use anyhow::{bail, Result};
use colored::Colorize;
use std::env;
use std::path::Path;

pub fn run(config: &Config, project: Option<&str>, message: Option<&str>) -> Result<()> {
    let dir = match project {
        Some(name) => projects::find(config, name)?,
        None => env::current_dir()?,
    };

    let repo = git::open_repo(&dir)?;

    render_if_available(config, &dir)?;

    if !git::has_changes(&repo)? {
        bail!("Nothing to snapshot (working tree clean)");
    }

    let label = message.unwrap_or("render");
    git::stage_and_commit(&repo, &format!("snapshot: {}", label))?;
    println!("{} Snapshot: {}", "◆".yellow(), label);

    Ok(())
}

/// Attempts a Reaper render if the project has a .rpp and reaper_command is configured.
/// Silently skips if either is absent — not all projects have a Reaper backend.
pub fn render_if_available(config: &Config, dir: &Path) -> Result<()> {
    let Some(reaper_cmd) = &config.reaper_command else {
        return Ok(());
    };
    match reaper::find_rpp(dir)? {
        Some(rpp) => reaper::render(reaper_cmd, &rpp),
        None => Ok(()),
    }
}
