use crate::git;
use anyhow::{bail, Context, Result};
use chrono::Local;
use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;

pub fn run(files: &[String], to: Option<&str>) -> Result<()> {
    if files.is_empty() {
        bail!("No files specified");
    }

    let cwd = env::current_dir()?;

    if !cwd.join("song.yaml").exists() {
        bail!("Not in a music-forge project (song.yaml not found)");
    }

    let repo = git::open_repo(&cwd)?;
    let mut index = repo.index()?;
    let today = Local::now().format("%Y-%m-%d").to_string();

    for file_str in files {
        let src = Path::new(file_str);
        if !src.exists() {
            bail!("File not found: {}", file_str);
        }

        let filename = src
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename")?;

        let subdir = to.unwrap_or_else(|| infer_subdir(filename));
        let dest_dir = cwd.join("assets").join(subdir);
        fs::create_dir_all(&dest_dir)?;

        let dest_name = format!("{}-{}", today, filename);
        let dest_path = dest_dir.join(&dest_name);

        fs::copy(src, &dest_path)
            .with_context(|| format!("Failed to copy {} to {}", file_str, dest_path.display()))?;

        // Stage the copied file
        let rel_path = dest_path.strip_prefix(&cwd).unwrap_or(&dest_path);
        index.add_path(rel_path)?;

        println!("{} {}", "Ingested:".green(), dest_path.display());
    }

    index.write()?;
    Ok(())
}

fn infer_subdir(filename: &str) -> &'static str {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "wav" | "flac" | "mp3" | "aif" | "aiff" | "ogg" | "aac" => "samples",
        "jpg" | "jpeg" | "png" | "pdf" => "references",
        _ => "samples",
    }
}
