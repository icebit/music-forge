use crate::git;
use anyhow::{bail, Result};
use colored::Colorize;
use std::env;

const VALID_STATUSES: &[&str] = &[
    "idea",
    "drafting",
    "arranging",
    "mixing",
    "mastering",
    "released",
    "abandoned",
];

pub fn run(status: &str) -> Result<()> {
    if !VALID_STATUSES.contains(&status) {
        bail!(
            "'{}' is not a valid status. Choose from: {}",
            status,
            VALID_STATUSES.join(", ")
        );
    }

    let cwd = env::current_dir()?;

    if !cwd.join("song.yaml").exists() {
        bail!("Not in a music-forge project (song.yaml not found)");
    }

    let repo = git::open_repo(&cwd)?;
    let tag_name = format!("status/{}", status);
    git::create_tag(&repo, &tag_name)?;

    println!("{} Status set to '{}'", "✓".green(), status);
    println!("  Tagged HEAD as '{}'", tag_name);

    Ok(())
}
