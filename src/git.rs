use anyhow::{bail, Context, Result};
use git2::{IndexAddOption, Repository, StatusOptions};
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
    let sig = repo.signature()?;
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
    opts.include_untracked(true).recurse_untracked_dirs(true);
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
