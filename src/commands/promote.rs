use crate::commands::init;
use crate::config::Config;
use anyhow::{bail, Result};
use colored::Colorize;
use std::path::Path;

pub fn run(config: &Config, seed_path: &str) -> Result<()> {
    let seed_dir = Path::new(seed_path);
    if !seed_dir.exists() {
        bail!("Seed path does not exist: {}", seed_path);
    }

    // Derive project name by stripping the YYYY-MM-DD- prefix from the directory name
    let basename = seed_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(seed_path);

    let name = strip_date_prefix(basename);
    let project_dir = config.projects_path().join(name);

    // Run full init workflow
    init::run(config, name)?;

    // Copy all seed contents into assets/ (skipping .git)
    copy_seed_contents(seed_dir, &project_dir.join("assets"))?;

    // Re-commit with the seed contents included
    let repo = crate::git::open_repo(&project_dir)?;
    crate::git::stage_and_commit(&repo, &format!("promote: seed contents from {}", basename))?;

    println!();
    println!("{} Remember to delete the seed when done:", "!".yellow());
    println!("  rm -rf {}", seed_dir.display());

    Ok(())
}

fn strip_date_prefix(name: &str) -> &str {
    // Matches YYYY-MM-DD- prefix (11 chars)
    if name.len() > 11 {
        let prefix = &name[..11];
        let looks_like_date = prefix.chars().enumerate().all(|(i, c)| match i {
            4 | 7 => c == '-',
            10 => c == '-',
            _ => c.is_ascii_digit(),
        });
        if looks_like_date {
            return &name[11..];
        }
    }
    name
}

fn copy_seed_contents(src: &Path, dest: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Skip the seed's git directory
        if name == ".git" {
            continue;
        }

        let dest_path = dest.join(&*name);
        let src_path = entry.path();

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dest.join(entry.file_name());
        let src_path = entry.path();
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
