use crate::config::Config;
use crate::git;
use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use git2::Repository;
use std::fs;
use std::path::Path;

pub fn run(config: &Config, name: &str) -> Result<()> {
    let seeds_dir = config.seeds_path();
    ensure_seeds_repo(&seeds_dir, &config.lfs_extensions)?;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let seed_dir = seeds_dir.join(format!("{}-{}", today, name));
    fs::create_dir_all(&seed_dir)?;

    let notes_md = format!("# {}\n", name);
    fs::write(seed_dir.join("notes.md"), notes_md)?;

    println!("{} Created seed '{}'", "✓".green(), name);
    println!("  {}", seed_dir.display());
    println!();
    println!("When ready to promote:");
    println!("  music-forge promote {}", seed_dir.display());

    Ok(())
}

fn ensure_seeds_repo(seeds_dir: &Path, lfs_extensions: &[String]) -> Result<()> {
    if seeds_dir.join(".git").exists() {
        return Ok(());
    }

    fs::create_dir_all(seeds_dir)?;
    let repo = Repository::init(seeds_dir)?;
    git::init_lfs(seeds_dir, lfs_extensions)?;

    // Create a README to make the initial commit non-empty
    fs::write(seeds_dir.join("README.md"), "# Seeds\n\nQuick ideas and sketches.\n")?;
    git::stage_and_commit(&repo, "init: seeds")?;

    println!("Initialized seeds repository at {}", seeds_dir.display());
    Ok(())
}
