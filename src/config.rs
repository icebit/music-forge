use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub projects_dir: String,
    pub seeds_dir: String,
    pub reaper_template: Option<String>,
    pub reaper_command: Option<String>,
    pub editor: Option<String>,
    #[serde(default = "default_debounce")]
    pub watch_debounce_seconds: u64,
    #[serde(default = "default_lfs_extensions")]
    pub lfs_extensions: Vec<String>,
}

fn default_debounce() -> u64 {
    300
}

fn default_lfs_extensions() -> Vec<String> {
    vec![
        "wav".into(),
        "flac".into(),
        "aif".into(),
        "aiff".into(),
        "mp3".into(),
        "ogg".into(),
        "aac".into(),
        "rpp".into(),
        "rpp-bak".into(),
    ]
}

impl Config {
    pub fn config_path() -> PathBuf {
        home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("music-forge")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::create_interactive()?;
            return Ok(config);
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| "Failed to parse config.toml")?;
        Ok(config)
    }

    pub fn create_interactive() -> Result<Self> {
        println!("No config found. Let's set up music-forge.");

        let projects_dir = prompt("Projects directory (e.g. ~/Music/Projects): ")?;
        let seeds_dir = prompt("Seeds directory (e.g. ~/Music/Seeds): ")?;
        let reaper_template = prompt_optional("Reaper template path (leave blank to skip): ")?;
        let reaper_command = prompt_optional("Reaper command (e.g. reaper, leave blank to skip): ")?;
        let editor = prompt_optional("Editor command (e.g. code, leave blank to skip): ")?;

        let config = Config {
            projects_dir: expand_tilde(&projects_dir),
            seeds_dir: expand_tilde(&seeds_dir),
            reaper_template: reaper_template.map(|s| expand_tilde(&s)),
            reaper_command,
            editor,
            watch_debounce_seconds: default_debounce(),
            lfs_extensions: default_lfs_extensions(),
        };

        let config_path = Self::config_path();
        std::fs::create_dir_all(config_path.parent().unwrap())?;
        let contents = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, contents)?;
        println!("Config saved to {}", config_path.display());

        Ok(config)
    }

    pub fn projects_path(&self) -> PathBuf {
        PathBuf::from(expand_tilde(&self.projects_dir))
    }

    pub fn seeds_path(&self) -> PathBuf {
        PathBuf::from(expand_tilde(&self.seeds_dir))
    }
}

pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = home_dir() {
            return format!("{}/{}", home.display(), &path[2..]);
        }
    }
    path.to_string()
}

fn prompt(label: &str) -> Result<String> {
    print!("{}", label);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_optional(label: &str) -> Result<Option<String>> {
    let val = prompt(label)?;
    if val.is_empty() {
        Ok(None)
    } else {
        Ok(Some(val))
    }
}
