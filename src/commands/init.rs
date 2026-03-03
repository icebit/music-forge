use crate::config::Config;
use crate::git;
use anyhow::{bail, Result};
use chrono::Local;
use colored::Colorize;
use git2::Repository;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(config: &Config, name: &str) -> Result<()> {
    let project_dir = config.projects_path().join(name);
    if project_dir.exists() {
        bail!("Project '{}' already exists at {}", name, project_dir.display());
    }

    create_directory_tree(&project_dir)?;
    let repo = init_git_repo(&project_dir, config)?;
    write_project_files(&project_dir, name)?;

    git::stage_and_commit(&repo, &format!("init: {}", name))?;
    println!("{} Created project '{}'", "✓".green(), name);
    println!("  {}", project_dir.display());

    open_tools(config, &project_dir, name);

    Ok(())
}

fn create_directory_tree(project_dir: &Path) -> Result<()> {
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

fn init_git_repo(project_dir: &Path, config: &Config) -> Result<Repository> {
    let repo = Repository::init(project_dir)?;
    git::init_lfs(project_dir, &config.lfs_extensions)?;
    Ok(repo)
}

fn write_project_files(project_dir: &Path, name: &str) -> Result<()> {
    let today = Local::now().format("%Y-%m-%d").to_string();

    let song_yaml = format!(
        "title: \"{}\"\ncreated: \"{}\"\ntags: []\ndescription: \"\"\n",
        name, today
    );
    fs::write(project_dir.join("song.yaml"), song_yaml)?;

    let notes_md = format!("# {}\n", name);
    fs::write(project_dir.join("notes.md"), notes_md)?;

    let gitignore = "*.reapeaks\n*.bwproject.cache/\n.DS_Store\nThumbs.db\n";
    fs::write(project_dir.join(".gitignore"), gitignore)?;

    Ok(())
}

fn open_tools(config: &Config, project_dir: &Path, name: &str) {
    let rpp_path = project_dir.join(format!("{}.rpp", name));

    // Copy reaper template if configured and available
    if let Some(template) = &config.reaper_template {
        let template_path = PathBuf::from(crate::config::expand_tilde(template));
        if template_path.exists() {
            if let Err(e) = fs::copy(&template_path, &rpp_path) {
                eprintln!("Warning: could not copy Reaper template: {}", e);
            }
        }
    }

    // Open Reaper if the .rpp exists and a command is configured
    if rpp_path.exists() {
        if let Some(reaper_cmd) = &config.reaper_command {
            Command::new(reaper_cmd)
                .arg(&rpp_path)
                .spawn()
                .ok();
        }
    }

    // Open editor if configured
    if let Some(editor_cmd) = &config.editor {
        Command::new(editor_cmd)
            .arg(project_dir)
            .spawn()
            .ok();
    }
}
