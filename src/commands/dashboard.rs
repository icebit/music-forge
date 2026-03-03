use crate::config::Config;
use crate::git;
use anyhow::Result;
use chrono::{DateTime, Local};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct SongYaml {
    title: String,
}

struct ProjectSummary {
    name: String,
    #[allow(dead_code)]
    title: String,
    commit_count: usize,
    last_activity: DateTime<Local>,
    current_status: Option<String>,
    status_since: Option<DateTime<Local>>,
}

pub fn run(config: &Config) -> Result<()> {
    let projects_dir = config.projects_path();
    if !projects_dir.exists() {
        anyhow::bail!(
            "Projects directory does not exist: {}",
            projects_dir.display()
        );
    }

    let mut summaries = collect_projects(&projects_dir)?;

    if summaries.is_empty() {
        println!("No projects found in {}", projects_dir.display());
        return Ok(());
    }

    summaries.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

    println!(
        "{}",
        format!(
            "{:<22} {:<13} {:<15} {:<10} {}",
            "PROJECT", "STATUS", "LAST ACTIVE", "COMMITS", "IN STAGE"
        )
        .bold()
    );
    println!("{}", "-".repeat(72));

    for s in &summaries {
        let status_str = s
            .current_status
            .as_deref()
            .map(|st| st.trim_start_matches("status/").to_string())
            .unwrap_or_else(|| "\u{2014}".to_string());

        let last_active = s.last_activity.format("%Y-%m-%d").to_string();

        let in_stage = s
            .status_since
            .map(human_duration)
            .unwrap_or_else(|| "\u{2014}".to_string());

        println!(
            "{:<22} {:<13} {:<15} {:<10} {}",
            s.name, status_str, last_active, s.commit_count, in_stage
        );
    }

    Ok(())
}

fn collect_projects(projects_dir: &Path) -> Result<Vec<ProjectSummary>> {
    let mut summaries = Vec::new();
    for entry in fs::read_dir(projects_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !path.join(".git").exists() || !path.join("song.yaml").exists() {
            continue;
        }
        match build_summary(&path) {
            Ok(summary) => summaries.push(summary),
            Err(e) => eprintln!(
                "Warning: skipping {}: {e}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
        }
    }
    Ok(summaries)
}

fn build_summary(project_path: &Path) -> Result<ProjectSummary> {
    let name = project_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let yaml_contents = fs::read_to_string(project_path.join("song.yaml"))?;
    let song: SongYaml = serde_yaml::from_str(&yaml_contents)?;

    let repo = git::open_repo(project_path)?;
    let commits = git::get_commit_log(&repo)?;

    let commit_count = commits.len();
    let last_activity = commits
        .first()
        .map(|c| c.time)
        .unwrap_or_else(Local::now);

    // Current status is the status tag on the most recent commit that has one
    let (current_status, status_since) = commits
        .iter()
        .find(|c| !c.status_tags.is_empty())
        .map(|c| {
            let tag = c.status_tags[0].clone();
            (Some(tag), Some(c.time))
        })
        .unwrap_or((None, None));

    Ok(ProjectSummary {
        name,
        title: song.title,
        commit_count,
        last_activity,
        current_status,
        status_since,
    })
}

fn human_duration(since: DateTime<Local>) -> String {
    let days = Local::now().signed_duration_since(since).num_days();
    if days < 1 {
        "today".to_string()
    } else if days < 7 {
        format!("{days}d")
    } else if days < 30 {
        format!("{}w", days / 7)
    } else if days < 365 {
        format!("{}mo", days / 30)
    } else {
        format!("{}y", days / 365)
    }
}
