use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;

const SHORT_HASH_LENGTH: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RepositoryStatus {
    pub branch: String,
    pub head: String,
    pub origin_url: Option<String>,
    pub changed_paths: usize,
    pub metadata_lock_exists: bool,
}

pub fn repository_status(root: &Path, metadata_path: &Path) -> Result<RepositoryStatus> {
    Ok(RepositoryStatus {
        branch: git_output(root, &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|_| "unknown".to_string()),
        head: git_output(
            root,
            &["rev-parse", &format!("--short={SHORT_HASH_LENGTH}"), "HEAD"],
        )
        .unwrap_or_else(|_| "unknown".to_string()),
        origin_url: git_output(root, &["config", "--get", "remote.origin.url"]).ok(),
        changed_paths: git_output(root, &["status", "--porcelain"])
            .map(|output| output.lines().count())
            .unwrap_or_default(),
        metadata_lock_exists: metadata_path.is_file(),
    })
}

pub fn repository_head(root: &Path) -> Result<String> {
    git_output(root, &["rev-parse", "HEAD"])
}

pub fn ls_remote_head(repo_url: &str) -> Result<String> {
    if !is_allowed_github_url(repo_url) {
        anyhow::bail!("unsupported origin URL: {repo_url}");
    }

    let output = Command::new("git")
        .args(["ls-remote", repo_url, "HEAD"])
        .output()
        .with_context(|| format!("failed to query origin {repo_url}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git ls-remote failed for {repo_url}: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .next()
        .map(str::to_string)
        .filter(|hash| is_hex_hash(hash))
        .with_context(|| format!("origin did not return a HEAD hash: {repo_url}"))
}

fn git_output(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {} failed: {stderr}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_allowed_github_url(url: &str) -> bool {
    let normalized = url.strip_suffix(".git").unwrap_or(url);
    normalized.starts_with("https://github.com/")
        && !normalized.chars().any(char::is_whitespace)
        && normalized.split('/').count() >= 5
}

fn is_hex_hash(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_https_github_urls_only() {
        assert!(is_allowed_github_url("https://github.com/ratatui/ratatui"));
        assert!(is_allowed_github_url(
            "https://github.com/ratatui/ratatui.git"
        ));
        assert!(!is_allowed_github_url("git@github.com:ratatui/ratatui.git"));
        assert!(!is_allowed_github_url("https://example.com/org/repo"));
    }
}
