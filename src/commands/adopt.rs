use crate::config::Config;
use crate::git;
use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use git2::Repository;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_all(config: &Config) -> Result<()> {
    let projects_path = config.projects_path();
    let mut dirs: Vec<PathBuf> = fs::read_dir(&projects_path)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();

    if dirs.is_empty() {
        println!("No directories found in {}", projects_path.display());
        return Ok(());
    }

    for dir in dirs {
        adopt_dir(config, &dir, false);
    }
    Ok(())
}

pub fn run(config: &Config) -> Result<()> {
    let project_dir = env::current_dir()?;
    adopt_dir(config, &project_dir, true);
    Ok(())
}

fn adopt_dir(config: &Config, project_dir: &Path, commit: bool) {
    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let result = (|| -> Result<()> {
        create_missing_dirs(project_dir)?;
        let repo = ensure_git_repo(project_dir, config)?;
        write_missing_files(project_dir, &name)?;
        if commit && git::has_changes(&repo)? {
            git::stage_and_commit(&repo, &format!("adopt: {}", name))?;
        }
        Ok(())
    })();

    match result {
        Ok(()) => println!("{} Adopted '{}'", "✓".green(), name),
        Err(e) => println!("{} '{}': {}", "✗".red(), name, e),
    }
}

fn create_missing_dirs(project_dir: &Path) -> Result<()> {
    for subdir in &[
        "assets/stems",
        "assets/samples",
        "assets/field-recordings",
        "assets/references",
        "exports",
        "promo",
    ] {
        fs::create_dir_all(project_dir.join(subdir))?;
    }
    Ok(())
}

fn ensure_git_repo(project_dir: &Path, config: &Config) -> Result<Repository> {
    let repo = Repository::init(project_dir)?;
    git::init_lfs(project_dir, &config.lfs_extensions)?;
    Ok(repo)
}

fn write_missing_files(project_dir: &Path, name: &str) -> Result<()> {
    let today = Local::now().format("%Y-%m-%d").to_string();

    let song_yaml_path = project_dir.join("song.yaml");
    if !song_yaml_path.exists() {
        fs::write(
            song_yaml_path,
            format!(
                "title: \"{}\"\ncreated: \"{}\"\ntags: []\ndescription: \"\"\n",
                name, today
            ),
        )?;
    }

    let notes_path = project_dir.join("notes.md");
    if !notes_path.exists() {
        fs::write(notes_path, format!("# {}\n", name))?;
    }

    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(
            gitignore_path,
            "*.reapeaks\n*.bwproject.cache/\n.DS_Store\nThumbs.db\n",
        )?;
    }

    Ok(())
}
