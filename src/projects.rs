use crate::config::Config;
use anyhow::{bail, Result};
use std::path::PathBuf;

/// Resolve a project or seed directory by exact or partial name match.
pub fn find(config: &Config, name: &str) -> Result<PathBuf> {
    let search_dirs = [config.projects_path(), config.seeds_path()];

    let mut matches: Vec<PathBuf> = Vec::new();
    for base in &search_dirs {
        if !base.exists() {
            continue;
        }
        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let query = name.to_lowercase();
            if dir_name == query || dir_name.contains(&query) {
                matches.push(path);
            }
        }
    }

    match matches.len() {
        0 => bail!("No project or seed matching '{}'", name),
        1 => Ok(matches.remove(0)),
        _ => {
            let names: Vec<String> = matches
                .iter()
                .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                .collect();
            bail!("Ambiguous — multiple matches for '{}':\n  {}", name, names.join("\n  "))
        }
    }
}
