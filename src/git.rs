use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{IndexAddOption, Oid, Repository, StatusOptions};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub fn open_repo(path: &Path) -> Result<Repository> {
    Repository::discover(path).with_context(|| {
        format!("Not a git repository (or any parent up to mount point): {}", path.display())
    })
}

pub fn stage_and_commit(repo: &Repository, message: &str) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let sig = repo.signature().with_context(|| {
        "Git identity not configured. Run:\n  git config --global user.name \"Your Name\"\n  git config --global user.email \"you@example.com\""
    })?;
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(_) => None,
    };

    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(())
}

pub fn has_changes(repo: &Repository) -> Result<bool> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    let statuses = repo.statuses(Some(&mut opts))?;
    Ok(!statuses.is_empty())
}

pub fn create_tag(repo: &Repository, name: &str) -> Result<()> {
    let head = repo.head()?.peel_to_commit()?;
    repo.tag_lightweight(name, head.as_object(), false)
        .with_context(|| format!("Failed to create tag '{}'", name))?;
    Ok(())
}

pub fn run_git_lfs(dir: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .arg("lfs")
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| "Failed to run git lfs (is git-lfs installed?)")?;
    if !status.success() {
        bail!("git lfs {} failed", args.join(" "));
    }
    Ok(())
}

pub fn init_lfs(dir: &Path, extensions: &[String]) -> Result<()> {
    run_git_lfs(dir, &["install"])?;
    for ext in extensions {
        run_git_lfs(dir, &["track", &format!("*.{}", ext)])?;
    }
    Ok(())
}

pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub time: DateTime<Local>,
    pub is_snapshot: bool,
    pub status_tags: Vec<String>,
}

// Walk all status/* tags and map each to the commit it points to.
pub fn get_status_tags_by_commit(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>> {
    let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
    let tag_names = repo.tag_names(None)?;
    for name in tag_names.iter().flatten() {
        if !name.starts_with("status/") {
            continue;
        }
        let obj = repo.revparse_single(name)?;
        let commit = obj.peel_to_commit()?;
        map.entry(commit.id()).or_default().push(name.to_string());
    }
    Ok(map)
}

// Walk commits from HEAD, annotating each with snapshot/status metadata.
// Returns an empty vec for repos with no commits.
pub fn get_commit_log(repo: &Repository) -> Result<Vec<CommitInfo>> {
    let tags_by_commit = get_status_tags_by_commit(repo)?;

    let mut revwalk = repo.revwalk()?;
    if revwalk.push_head().is_err() {
        return Ok(Vec::new());
    }
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let message = commit.summary().unwrap_or("").to_string();
        let seconds = commit.author().when().seconds();
        let time = Local
            .timestamp_opt(seconds, 0)
            .single()
            .unwrap_or_else(|| Local::now());
        let is_snapshot = message.starts_with("snapshot:");
        let status_tags = tags_by_commit.get(&oid).cloned().unwrap_or_default();
        commits.push(CommitInfo {
            hash: oid.to_string()[..7].to_string(),
            message,
            time,
            is_snapshot,
            status_tags,
        });
    }
    Ok(commits)
}
