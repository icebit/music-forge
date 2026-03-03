use crate::git;
use anyhow::{bail, Result};
use colored::Colorize;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct SongYaml {
    title: String,
}

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let song_yaml_path = cwd.join("song.yaml");
    if !song_yaml_path.exists() {
        bail!("Not a music-forge project (no song.yaml in current directory)");
    }

    let yaml_contents = fs::read_to_string(&song_yaml_path)?;
    let song: SongYaml = serde_yaml::from_str(&yaml_contents)?;

    let repo = git::open_repo(&cwd)?;
    let commits = git::get_commit_log(&repo)?;

    println!("{}", format!("{}  ({} commits)", song.title, commits.len()).bold());
    println!();

    for commit in &commits {
        let timestamp = commit.time.format("%Y-%m-%d %H:%M").to_string();
        let base = format!("{timestamp}  {}  {}", commit.hash, commit.message);

        // Build the styled line: snapshot lines are bold yellow and get a [snapshot] label
        let mut line = if commit.is_snapshot {
            format!("{base}  [snapshot]").yellow().bold().to_string()
        } else {
            base
        };

        for tag in &commit.status_tags {
            line.push_str(&format!("  {}", format!("[{tag}]").cyan()));
        }

        println!("{line}");
    }

    Ok(())
}
