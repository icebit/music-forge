use crate::config::Config;
use crate::projects;
use anyhow::Result;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

const DAW_EXTENSIONS: &[&str] = &["rpp", "bwproject", "logicx", "song"];

pub fn run(config: &Config, name: &str) -> Result<()> {
    let project_dir = projects::find(config, name)?;
    let display_name = project_dir.file_name().unwrap_or_default().to_string_lossy();

    // Open DAW file with system default app
    if let Some(daw_file) = find_daw_file(&project_dir)? {
        Command::new("open").arg(&daw_file).spawn().ok();
        println!(
            "{} {}  ({})",
            "Opening:".green(),
            display_name,
            daw_file.file_name().unwrap_or_default().to_string_lossy()
        );
    } else {
        println!("{} {}  (no DAW file found)", "Opening:".green(), display_name);
    }

    // Open editor if configured
    if let Some(editor_cmd) = &config.editor {
        Command::new(editor_cmd).arg(&project_dir).spawn().ok();
    }

    Ok(())
}

fn find_daw_file(dir: &Path) -> Result<Option<PathBuf>> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| DAW_EXTENSIONS.contains(&e.to_lowercase().as_str()))
                    .unwrap_or(false)
        })
        .collect();

    // Prefer the shortest filename — most likely the "canonical" one vs numbered saves
    entries.sort_by_key(|p| {
        p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .len()
    });

    Ok(entries.into_iter().next())
}
