use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::default_metadata_path;
use crate::git_utils;

const README_FILE: &str = "README.md";
const SKILL_FILE_NAME: &str = "SKILL.md";
const LICENSE_SNIPPET_CHAR_LIMIT: usize = 180;
const MIN_COMMIT_LENGTH: usize = 7;
const MAX_COMMIT_LENGTH: usize = 40;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncOptions {
    pub root: PathBuf,
    pub output: PathBuf,
    pub refresh_origin: bool,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillLockfile {
    pub schema_version: u16,
    pub generated_unix_seconds: u64,
    pub source_repo: String,
    pub skills: Vec<SkillMetadata>,
    pub hooks: Vec<HookMetadata>,
    pub mcp_servers: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: Option<String>,
    pub local_path: String,
    pub collection: String,
    pub license: LicenseMetadata,
    pub version: VersionMetadata,
    pub origin: OriginMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LicenseMetadata {
    pub spdx: Option<String>,
    pub source: String,
    pub file: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub value: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OriginMetadata {
    pub source_url: Option<String>,
    pub repo_url: Option<String>,
    pub declared_commit: Option<String>,
    pub head_commit: Option<String>,
    pub head_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HookMetadata {
    pub name: String,
    pub local_path: String,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncReport {
    pub output: PathBuf,
    pub skill_count: usize,
    pub missing_license_count: usize,
    pub missing_version_count: usize,
    pub origin_error_count: usize,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
    license: Option<String>,
    version: Option<String>,
    origin_url: Option<String>,
    origin_commit: Option<String>,
    last_synced_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Attribution {
    path_patterns: Vec<String>,
    source_url: Option<String>,
    license: Option<String>,
    declared_commit: Option<String>,
}

pub fn sync_metadata(options: &SyncOptions) -> Result<SyncReport> {
    let lockfile = build_lockfile(&options.root, options.refresh_origin)?;
    let report = SyncReport {
        output: options.output.clone(),
        skill_count: lockfile.skills.len(),
        missing_license_count: lockfile
            .skills
            .iter()
            .filter(|skill| skill.license.spdx.as_deref() == Some("NOASSERTION"))
            .count(),
        missing_version_count: lockfile
            .skills
            .iter()
            .filter(|skill| skill.version.value.is_none())
            .count(),
        origin_error_count: lockfile
            .skills
            .iter()
            .filter(|skill| skill.origin.head_error.is_some())
            .count(),
        dry_run: options.dry_run,
    };

    if options.dry_run {
        return Ok(report);
    }

    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let serialized =
        serde_json::to_string_pretty(&lockfile).context("failed to serialize metadata lockfile")?;
    fs::write(&options.output, format!("{serialized}\n"))
        .with_context(|| format!("failed to write {}", options.output.display()))?;

    Ok(report)
}

pub fn default_sync_options(root: PathBuf) -> SyncOptions {
    SyncOptions {
        output: default_metadata_path(&root),
        root,
        refresh_origin: false,
        dry_run: false,
    }
}

fn build_lockfile(root: &Path, refresh_origin: bool) -> Result<SkillLockfile> {
    let attributions = parse_readme_attributions(root)?;
    let skill_dirs = discover_skill_dirs(root)?;
    let local_commit = git_utils::repository_head(root).ok();
    let origin_heads = if refresh_origin {
        refresh_origin_heads(&skill_dirs, &attributions, root)?
    } else {
        BTreeMap::new()
    };

    let mut skills = Vec::with_capacity(skill_dirs.len());
    for skill_dir in skill_dirs {
        skills.push(build_skill_metadata(
            root,
            &skill_dir,
            &attributions,
            &origin_heads,
            local_commit.as_deref(),
        )?);
    }

    let hooks = discover_hooks(root)?;
    let mcp_servers = discover_mcp_servers(root)?;

    Ok(SkillLockfile {
        schema_version: 1,
        generated_unix_seconds: unix_seconds_now(),
        source_repo: "stacc".to_string(),
        skills,
        hooks,
        mcp_servers,
    })
}

fn build_skill_metadata(
    root: &Path,
    skill_dir: &Path,
    attributions: &[Attribution],
    origin_heads: &BTreeMap<String, Result<String, String>>,
    local_commit: Option<&str>,
) -> Result<SkillMetadata> {
    let skill_path = skill_dir.join(SKILL_FILE_NAME);
    let frontmatter = parse_skill_frontmatter(&skill_path)?;
    let relative_path = relative_string(root, skill_dir)?;
    let attribution = attributions
        .iter()
        .filter(|candidate| attribution_matches(candidate, &relative_path))
        .max_by_key(|candidate| attribution_score(candidate, &relative_path));
    let license = resolve_license(root, skill_dir, &frontmatter, attribution)?;

    let source_url = frontmatter
        .origin_url
        .clone()
        .or_else(|| attribution.and_then(|value| value.source_url.clone()));
    let repo_url = source_url.as_deref().and_then(github_repo_url);
    let declared_commit = frontmatter
        .origin_commit
        .clone()
        .or_else(|| attribution.and_then(|value| value.declared_commit.clone()));
    let (head_commit, head_error) = repo_url
        .as_ref()
        .and_then(|url| origin_heads.get(url))
        .map(|result| match result {
            Ok(commit) => (Some(commit.clone()), None),
            Err(error) => (None, Some(error.clone())),
        })
        .unwrap_or((None, None));

    let version = resolve_version(
        &frontmatter,
        declared_commit.as_ref(),
        head_commit.as_ref(),
        local_commit,
    );

    Ok(SkillMetadata {
        name: frontmatter.name.clone().unwrap_or_else(|| {
            skill_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        }),
        description: frontmatter.description.clone(),
        local_path: relative_path.clone(),
        collection: collection_for_path(&relative_path),
        license,
        version,
        origin: OriginMetadata {
            source_url,
            repo_url,
            declared_commit,
            head_commit,
            head_error,
        },
    })
}

fn resolve_license(
    root: &Path,
    skill_dir: &Path,
    frontmatter: &SkillFrontmatter,
    attribution: Option<&Attribution>,
) -> Result<LicenseMetadata> {
    if let Some(license) = frontmatter
        .license
        .as_deref()
        .and_then(normalize_license_value)
    {
        return Ok(LicenseMetadata {
            spdx: Some(license),
            source: "frontmatter".to_string(),
            file: license_file_path(root, skill_dir),
        });
    }

    if let Some(path) = license_file_path(root, skill_dir) {
        let license_text = fs::read_to_string(root.join(&path))
            .with_context(|| format!("failed to read license file {}", path))?;
        return Ok(LicenseMetadata {
            spdx: infer_license_spdx(&license_text),
            source: "license-file".to_string(),
            file: Some(path),
        });
    }

    if let Some(license) = attribution
        .and_then(|value| value.license.as_deref())
        .and_then(normalize_license_value)
    {
        return Ok(LicenseMetadata {
            spdx: Some(license),
            source: "readme-attribution".to_string(),
            file: None,
        });
    }

    Ok(LicenseMetadata {
        spdx: Some("NOASSERTION".to_string()),
        source: "missing".to_string(),
        file: None,
    })
}

fn resolve_version(
    frontmatter: &SkillFrontmatter,
    declared_commit: Option<&String>,
    head_commit: Option<&String>,
    local_commit: Option<&str>,
) -> VersionMetadata {
    if let Some(version) = frontmatter.version.clone() {
        return VersionMetadata {
            value: Some(version),
            source: "frontmatter".to_string(),
        };
    }

    if let Some(commit) = declared_commit {
        return VersionMetadata {
            value: Some(format!("git:{commit}")),
            source: "declared-origin-commit".to_string(),
        };
    }

    if let Some(commit) = head_commit {
        return VersionMetadata {
            value: Some(format!("git:{commit}")),
            source: "origin-head".to_string(),
        };
    }

    if frontmatter.last_synced_at.is_some() {
        return VersionMetadata {
            value: frontmatter.last_synced_at.clone(),
            source: "last-synced-at".to_string(),
        };
    }

    if let Some(commit) = local_commit {
        return VersionMetadata {
            value: Some(format!("local-git:{commit}")),
            source: "local-repo-head".to_string(),
        };
    }

    VersionMetadata {
        value: None,
        source: "missing".to_string(),
    }
}

fn parse_skill_frontmatter(path: &Path) -> Result<SkillFrontmatter> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut lines = contents.lines();
    if lines.next() != Some("---") {
        return Ok(empty_frontmatter());
    }

    let mut frontmatter_lines = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        frontmatter_lines.push(line.to_string());
    }

    Ok(parse_frontmatter_lines(&frontmatter_lines))
}

fn parse_frontmatter_lines(lines: &[String]) -> SkillFrontmatter {
    let mut frontmatter = empty_frontmatter();
    let mut active_multiline_key: Option<String> = None;
    let mut active_multiline_value = String::new();

    for line in lines {
        if let Some(key) = active_multiline_key.clone() {
            if line.starts_with(' ') || line.starts_with('\t') || line.trim().is_empty() {
                if !active_multiline_value.is_empty() {
                    active_multiline_value.push(' ');
                }
                active_multiline_value.push_str(line.trim());
                continue;
            }
            assign_frontmatter_value(&mut frontmatter, &key, active_multiline_value.trim());
            active_multiline_key = None;
            active_multiline_value.clear();
        }

        let Some((key, raw_value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = raw_value.trim().trim_matches('"');
        if value.starts_with('>') || value.starts_with('|') {
            active_multiline_key = Some(key.to_string());
            active_multiline_value.clear();
            continue;
        }
        assign_frontmatter_value(&mut frontmatter, key, value);
    }

    if let Some(key) = active_multiline_key {
        assign_frontmatter_value(&mut frontmatter, &key, active_multiline_value.trim());
    }

    frontmatter
}

fn assign_frontmatter_value(frontmatter: &mut SkillFrontmatter, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }

    match key {
        "name" => frontmatter.name = Some(value.to_string()),
        "description" => frontmatter.description = Some(value.to_string()),
        "license" => frontmatter.license = Some(value.to_string()),
        "version" | "metadata.version" => frontmatter.version = Some(value.to_string()),
        "origin_url" | "metadata.origin_url" => frontmatter.origin_url = Some(value.to_string()),
        "origin_commit" | "metadata.origin_commit" => {
            frontmatter.origin_commit = Some(value.to_string())
        }
        "last_synced_at" | "metadata.last_synced_at" => {
            frontmatter.last_synced_at = Some(value.to_string())
        }
        _ => {}
    }
}

fn empty_frontmatter() -> SkillFrontmatter {
    SkillFrontmatter {
        name: None,
        description: None,
        license: None,
        version: None,
        origin_url: None,
        origin_commit: None,
        last_synced_at: None,
    }
}

fn parse_readme_attributions(root: &Path) -> Result<Vec<Attribution>> {
    let path = root.join(README_FILE);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut attributions = Vec::new();
    for line in contents.lines() {
        if !line.starts_with('|') || line.starts_with("|------") || line.contains("| File |") {
            continue;
        }
        let cells = split_markdown_table_row(line);
        if cells.len() < 5 {
            continue;
        }
        let path_patterns = extract_backtick_values(&cells[0]);
        if path_patterns.is_empty() {
            continue;
        }
        let notes = &cells[2];
        attributions.push(Attribution {
            path_patterns,
            source_url: extract_markdown_link_url(&cells[3]),
            license: normalize_markdown_text(&cells[4]),
            declared_commit: find_commit_hash(notes),
        });
    }
    Ok(attributions)
}

fn split_markdown_table_row(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn extract_backtick_values(value: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut remaining = value;
    while let Some(start) = remaining.find('`') {
        let after_start = &remaining[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        values.push(after_start[..end].trim_end_matches('/').to_string());
        remaining = &after_start[end + 1..];
    }
    values
}

fn extract_markdown_link_url(value: &str) -> Option<String> {
    let start = value.find("](")?;
    let after_start = &value[start + 2..];
    let end = after_start.find(')')?;
    Some(after_start[..end].to_string())
}

fn normalize_markdown_text(value: &str) -> Option<String> {
    let text = value
        .replace("`", "")
        .replace("<br>", " ")
        .trim()
        .to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn find_commit_hash(value: &str) -> Option<String> {
    value
        .split(|ch: char| !ch.is_ascii_hexdigit())
        .find(|token| {
            (MIN_COMMIT_LENGTH..=MAX_COMMIT_LENGTH).contains(&token.len())
                && token.chars().all(|ch| ch.is_ascii_hexdigit())
        })
        .map(str::to_string)
}

fn attribution_matches(attribution: &Attribution, relative_path: &str) -> bool {
    attribution
        .path_patterns
        .iter()
        .any(|pattern| path_pattern_matches(pattern, relative_path))
}

fn attribution_score(attribution: &Attribution, relative_path: &str) -> usize {
    attribution
        .path_patterns
        .iter()
        .filter(|pattern| path_pattern_matches(pattern, relative_path))
        .map(|pattern| pattern.replace('*', "").len())
        .max()
        .unwrap_or(relative_path.len())
}

fn path_pattern_matches(pattern: &str, relative_path: &str) -> bool {
    let normalized = pattern.trim_end_matches('/');
    if let Some((prefix, suffix)) = normalized.split_once('*') {
        return relative_path.starts_with(prefix) && relative_path.ends_with(suffix);
    }
    relative_path == normalized || relative_path.starts_with(&format!("{normalized}/"))
}

fn discover_skill_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = BTreeSet::new();
    let roots = [
        root.join("configs").join("skills"),
        root.join("configs").join("codex-skills").join("skills"),
        root.join("configs").join("cursor-plugins").join("skills"),
        root.join("configs").join("stacks"),
    ];
    for path in roots {
        collect_skill_dirs(&path, &mut dirs)?;
    }
    Ok(dirs.into_iter().collect())
}

fn collect_skill_dirs(path: &Path, dirs: &mut BTreeSet<PathBuf>) -> Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_skill_dirs(&entry_path, dirs)?;
        } else if entry.file_name() == SKILL_FILE_NAME {
            if let Some(parent) = entry_path.parent() {
                dirs.insert(parent.to_path_buf());
            }
        }
    }
    Ok(())
}

fn discover_hooks(root: &Path) -> Result<Vec<HookMetadata>> {
    let mut hooks = Vec::new();
    append_hooks(
        root,
        &root.join("configs").join("hooks"),
        "generic",
        &mut hooks,
    )?;
    append_hooks(
        root,
        &root.join("configs").join("cursor-plugins").join("hooks"),
        "cursor-plugin",
        &mut hooks,
    )?;
    hooks.sort_by(|left, right| left.local_path.cmp(&right.local_path));
    Ok(hooks)
}

fn append_hooks(
    root: &Path,
    parent: &Path,
    source: &str,
    hooks: &mut Vec<HookMetadata>,
) -> Result<()> {
    if !parent.is_dir() {
        return Ok(());
    }

    for entry in
        fs::read_dir(parent).with_context(|| format!("failed to read {}", parent.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        hooks.push(HookMetadata {
            name: entry.file_name().to_string_lossy().into_owned(),
            local_path: relative_string(root, &path)?,
            source: source.to_string(),
        });
    }

    Ok(())
}

fn discover_mcp_servers(root: &Path) -> Result<Vec<String>> {
    let path = root.join("configs").join("mcps").join("mcp.json");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&contents)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    let mut servers = value
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
        .map(|map| map.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    servers.sort();
    Ok(servers)
}

fn refresh_origin_heads(
    skill_dirs: &[PathBuf],
    attributions: &[Attribution],
    root: &Path,
) -> Result<BTreeMap<String, Result<String, String>>> {
    let mut repo_urls = BTreeSet::new();
    for skill_dir in skill_dirs {
        let relative_path = relative_string(root, skill_dir)?;
        let frontmatter = parse_skill_frontmatter(&skill_dir.join(SKILL_FILE_NAME))?;
        let attribution = attributions
            .iter()
            .filter(|candidate| attribution_matches(candidate, &relative_path))
            .max_by_key(|candidate| attribution_score(candidate, &relative_path));
        let source_url = frontmatter
            .origin_url
            .or_else(|| attribution.and_then(|value| value.source_url.clone()));
        if let Some(repo_url) = source_url.as_deref().and_then(github_repo_url) {
            repo_urls.insert(repo_url);
        }
    }

    let mut heads = BTreeMap::new();
    for repo_url in repo_urls {
        let result = git_utils::ls_remote_head(&repo_url).map_err(|error| error.to_string());
        heads.insert(repo_url, result);
    }
    Ok(heads)
}

fn license_file_path(root: &Path, skill_dir: &Path) -> Option<String> {
    ["LICENSE.txt", "LICENSE", "LICENSE.md"]
        .iter()
        .map(|name| skill_dir.join(name))
        .find(|path| path.is_file())
        .and_then(|path| relative_string(root, &path).ok())
}

fn infer_license_spdx(text: &str) -> Option<String> {
    let snippet = text
        .chars()
        .take(LICENSE_SNIPPET_CHAR_LIMIT)
        .collect::<String>();
    let normalized = snippet.to_ascii_lowercase();
    if normalized.contains("mit license") {
        return Some("MIT".to_string());
    }
    if normalized.contains("apache license") && normalized.contains("version 2.0") {
        return Some("Apache-2.0".to_string());
    }
    if normalized.contains("creative commons zero") || normalized.contains("cc0") {
        return Some("CC0-1.0".to_string());
    }
    None
}

fn normalize_license_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    match trimmed {
        "" | "Complete terms in LICENSE.txt" => None,
        "MIT" | "Apache-2.0" | "CC0-1.0" | "BSD-3-Clause" | "BSD-2-Clause" => {
            Some(trimmed.to_string())
        }
        _ => Some(trimmed.to_string()),
    }
}

fn github_repo_url(source_url: &str) -> Option<String> {
    let without_suffix = source_url.strip_suffix(".git").unwrap_or(source_url);
    let marker = "https://github.com/";
    let after_marker = without_suffix.strip_prefix(marker)?;
    let mut parts = after_marker.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some(format!("{marker}{owner}/{repo}"))
}

fn collection_for_path(relative_path: &str) -> String {
    if relative_path.starts_with("configs/codex-skills/") {
        "codex-skills".to_string()
    } else if relative_path.starts_with("configs/cursor-plugins/") {
        "cursor-plugins".to_string()
    } else if relative_path.starts_with("configs/stacks/") {
        "stack".to_string()
    } else {
        "skills".to_string()
    }
}

fn relative_string(root: &Path, path: &Path) -> Result<String> {
    Ok(path
        .strip_prefix(root)
        .with_context(|| format!("{} is not under {}", path.display(), root.display()))?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn unix_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_commit_hash_from_notes() {
        assert_eq!(
            find_commit_hash("Copied from path at `956a92b`; adapted"),
            Some("956a92b".to_string())
        );
    }

    #[test]
    fn matches_wildcard_paths() {
        assert!(path_pattern_matches(
            "configs/skills/expo-*",
            "configs/skills/expo-module"
        ));
        assert!(!path_pattern_matches(
            "configs/skills/expo-*",
            "configs/skills/tdd"
        ));
    }

    #[test]
    fn normalizes_github_tree_url_to_repo() {
        assert_eq!(
            github_repo_url("https://github.com/expo/skills/tree/main/plugins/expo/skills"),
            Some("https://github.com/expo/skills".to_string())
        );
    }
}
