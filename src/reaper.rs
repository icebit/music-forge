use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_rpp(dir: &Path) -> Result<Option<PathBuf>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rpp") {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

pub fn render(reaper_cmd: &str, rpp_path: &Path) -> Result<()> {
    println!("Rendering {}...", rpp_path.file_name().unwrap_or_default().to_string_lossy());
    let status = Command::new(reaper_cmd)
        .args(["-nosplash", "-nonewinst", "-renderproject"])
        .arg(rpp_path)
        .status()
        .with_context(|| format!("Failed to run Reaper ('{}')", reaper_cmd))?;
    if !status.success() {
        bail!("Reaper render failed (exit code: {})", status.code().unwrap_or(-1));
    }
    Ok(())
}
