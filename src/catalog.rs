use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_METADATA_PATH: &str = "configs/metadata/skills.lock.json";
pub const DEFAULT_PANEL_CONFIG_PATH: &str = "configs/stacc-panel.json";

const CONFIGS_DIR: &str = "configs";
const CODEX_PLUGINS_CONFIG_FILE: &str = "plugins.json";
const CODEX_PLUGINS_KEY: &str = "plugins";
const SKILL_FILE_NAME: &str = "SKILL.md";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Editor {
    Cursor,
    Claude,
    Opencode,
    Codex,
    Ampcode,
}

impl Editor {
    pub const ALL: [Editor; 5] = [
        Editor::Cursor,
        Editor::Claude,
        Editor::Opencode,
        Editor::Codex,
        Editor::Ampcode,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Editor::Cursor => "Cursor",
            Editor::Claude => "Claude Code",
            Editor::Opencode => "OpenCode",
            Editor::Codex => "Codex",
            Editor::Ampcode => "AMP Code",
        }
    }
}

impl fmt::Display for Editor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Editor::Cursor => "cursor",
            Editor::Claude => "claude",
            Editor::Opencode => "opencode",
            Editor::Codex => "codex",
            Editor::Ampcode => "ampcode",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    Global,
    Project,
}

impl fmt::Display for Scope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Scope::Global => "global",
            Scope::Project => "project",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Commands,
    Rules,
    Agents,
    Skills,
    Stack,
    Hooks,
    Mcps,
    CursorPlugins,
    CodexSkills,
    CodexPlugins,
}

impl Category {
    pub const ALL: [Category; 10] = [
        Category::Commands,
        Category::Rules,
        Category::Agents,
        Category::Skills,
        Category::Stack,
        Category::Hooks,
        Category::Mcps,
        Category::CursorPlugins,
        Category::CodexSkills,
        Category::CodexPlugins,
    ];

    pub fn install_value(self) -> &'static str {
        match self {
            Category::Commands => "commands",
            Category::Rules => "rules",
            Category::Agents => "agents",
            Category::Skills => "skills",
            Category::Stack => "stack",
            Category::Hooks => "hooks",
            Category::Mcps => "mcps",
            Category::CursorPlugins => "cursor-plugins",
            Category::CodexSkills => "codex-skills",
            Category::CodexPlugins => "codex-plugins",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Commands => "Commands",
            Category::Rules => "Rules",
            Category::Agents => "Agents",
            Category::Skills => "Skills",
            Category::Stack => "Stacks",
            Category::Hooks => "Hooks",
            Category::Mcps => "MCPs",
            Category::CursorPlugins => "Cursor plugins",
            Category::CodexSkills => "Codex skills",
            Category::CodexPlugins => "Codex plugins",
        }
    }

    pub fn is_supported_for(self, editor: Editor, scope: Scope, root: &Path) -> bool {
        if !self.source_exists(root) {
            return false;
        }
        if self == Category::Hooks {
            return match editor {
                Editor::Cursor => hooks_source_exists(root),
                Editor::Claude => root.join(CONFIGS_DIR).join("hooks").is_dir(),
                Editor::Opencode | Editor::Codex | Editor::Ampcode => false,
            };
        }

        match editor {
            Editor::Cursor => matches!(
                self,
                Category::Commands
                    | Category::Rules
                    | Category::Agents
                    | Category::Skills
                    | Category::Stack
                    | Category::Hooks
                    | Category::Mcps
                    | Category::CursorPlugins
            ),
            Editor::Claude => matches!(
                self,
                Category::Commands
                    | Category::Rules
                    | Category::Agents
                    | Category::Skills
                    | Category::Stack
                    | Category::Hooks
                    | Category::Mcps
            ),
            Editor::Opencode => matches!(
                self,
                Category::Commands
                    | Category::Rules
                    | Category::Agents
                    | Category::Skills
                    | Category::Stack
                    | Category::Mcps
            ),
            Editor::Codex => match scope {
                Scope::Global => matches!(
                    self,
                    Category::Commands
                        | Category::Rules
                        | Category::Skills
                        | Category::Stack
                        | Category::Mcps
                        | Category::CodexSkills
                        | Category::CodexPlugins
                ),
                Scope::Project => matches!(
                    self,
                    Category::Commands
                        | Category::Rules
                        | Category::Skills
                        | Category::Stack
                        | Category::CodexSkills
                ),
            },
            Editor::Ampcode => match scope {
                Scope::Global => matches!(
                    self,
                    Category::Commands
                        | Category::Rules
                        | Category::Skills
                        | Category::Stack
                        | Category::Mcps
                ),
                Scope::Project => matches!(
                    self,
                    Category::Commands | Category::Rules | Category::Skills | Category::Stack
                ),
            },
        }
    }

    pub fn source_exists(self, root: &Path) -> bool {
        match self {
            Category::Stack => root.join(CONFIGS_DIR).join("stacks").is_dir(),
            Category::Hooks => hooks_source_exists(root),
            Category::CursorPlugins => root.join(CONFIGS_DIR).join("cursor-plugins").is_dir(),
            Category::CodexSkills => root.join(CONFIGS_DIR).join("codex-skills").is_dir(),
            Category::CodexPlugins => root.join(CONFIGS_DIR).join("codex-plugins").is_dir(),
            _ => root.join(CONFIGS_DIR).join(self.install_value()).is_dir(),
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.install_value())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictMode {
    Overwrite,
    Backup,
    Skip,
    Selective,
}

impl ConflictMode {
    pub fn install_value(self) -> &'static str {
        match self {
            ConflictMode::Overwrite => "overwrite",
            ConflictMode::Backup => "backup",
            ConflictMode::Skip => "skip",
            ConflictMode::Selective => "selective",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ConflictMode::Overwrite => ConflictMode::Backup,
            ConflictMode::Backup => ConflictMode::Skip,
            ConflictMode::Skip => ConflictMode::Selective,
            ConflictMode::Selective => ConflictMode::Overwrite,
        }
    }
}

impl fmt::Display for ConflictMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.install_value())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HookPackage {
    pub name: String,
    pub path: PathBuf,
    pub source: HookSource,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum HookSource {
    Generic,
    CursorPlugin,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Catalog {
    pub root: PathBuf,
    pub categories: Vec<Category>,
    pub stacks: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub hook_packages: Vec<HookPackage>,
    pub codex_plugins: Vec<String>,
    pub skill_count: usize,
}

pub fn discover_catalog(root: &Path) -> Result<Catalog> {
    let categories = Category::ALL
        .into_iter()
        .filter(|category| category.source_exists(root))
        .collect::<Vec<_>>();

    Ok(Catalog {
        root: root.to_path_buf(),
        categories,
        stacks: discover_child_dirs(&root.join(CONFIGS_DIR).join("stacks"))?,
        mcp_servers: discover_mcp_servers(root)?,
        hook_packages: discover_hook_packages(root)?,
        codex_plugins: discover_codex_plugins(root)?,
        skill_count: count_skill_files(root)?,
    })
}

pub fn default_metadata_path(root: &Path) -> PathBuf {
    root.join(DEFAULT_METADATA_PATH)
}

pub fn default_panel_config_path(root: &Path) -> PathBuf {
    root.join(DEFAULT_PANEL_CONFIG_PATH)
}

pub fn repo_root_from_option(root: Option<PathBuf>) -> Result<PathBuf> {
    let candidate = match root {
        Some(path) => path,
        None => std::env::current_dir().context("failed to read current directory")?,
    };
    let configs = candidate.join(CONFIGS_DIR);
    if !configs.is_dir() {
        anyhow::bail!("{} does not contain configs/", candidate.display());
    }
    candidate
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", candidate.display()))
}

fn discover_child_dirs(parent: &Path) -> Result<Vec<String>> {
    if !parent.is_dir() {
        return Ok(Vec::new());
    }

    let mut children = Vec::new();
    for entry in
        fs::read_dir(parent).with_context(|| format!("failed to read {}", parent.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            children.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    children.sort();
    Ok(children)
}

fn discover_mcp_servers(root: &Path) -> Result<Vec<String>> {
    let path = root.join(CONFIGS_DIR).join("mcps").join("mcp.json");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let value = read_json(&path)?;
    let mut servers = value
        .get("mcpServers")
        .and_then(Value::as_object)
        .map(|servers| servers.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    servers.sort();
    Ok(servers)
}

fn discover_codex_plugins(root: &Path) -> Result<Vec<String>> {
    let path = root
        .join(CONFIGS_DIR)
        .join("codex-plugins")
        .join(CODEX_PLUGINS_CONFIG_FILE);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let value = read_json(&path)?;
    let mut plugins = value
        .get(CODEX_PLUGINS_KEY)
        .and_then(Value::as_object)
        .map(|plugins| plugins.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    plugins.sort();
    Ok(plugins)
}

fn discover_hook_packages(root: &Path) -> Result<Vec<HookPackage>> {
    let mut packages = Vec::new();
    let generic_hooks = root.join(CONFIGS_DIR).join("hooks");
    append_hook_packages(&mut packages, &generic_hooks, HookSource::Generic)?;

    let cursor_hooks = root.join(CONFIGS_DIR).join("cursor-plugins").join("hooks");
    append_hook_packages(&mut packages, &cursor_hooks, HookSource::CursorPlugin)?;

    packages.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(packages)
}

fn hooks_source_exists(root: &Path) -> bool {
    root.join(CONFIGS_DIR).join("hooks").is_dir()
        || root
            .join(CONFIGS_DIR)
            .join("cursor-plugins")
            .join("hooks")
            .is_dir()
}

fn append_hook_packages(
    packages: &mut Vec<HookPackage>,
    parent: &Path,
    source: HookSource,
) -> Result<()> {
    if !parent.is_dir() {
        return Ok(());
    }

    for name in discover_child_dirs(parent)? {
        packages.push(HookPackage {
            name: name.clone(),
            path: parent.join(&name),
            source: source.clone(),
        });
    }

    Ok(())
}

fn count_skill_files(root: &Path) -> Result<usize> {
    let mut paths = BTreeSet::new();
    let roots = [
        root.join(CONFIGS_DIR).join("skills"),
        root.join(CONFIGS_DIR).join("codex-skills").join("skills"),
        root.join(CONFIGS_DIR).join("cursor-plugins").join("skills"),
        root.join(CONFIGS_DIR).join("stacks"),
    ];
    for path in roots {
        collect_skill_files(&path, &mut paths)?;
    }
    Ok(paths.len())
}

fn collect_skill_files(path: &Path, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_skill_files(&entry_path, paths)?;
        } else if entry.file_name() == SKILL_FILE_NAME {
            paths.insert(entry_path);
        }
    }

    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("invalid JSON in {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_project_does_not_support_mcp() {
        let root = Path::new(".");
        assert!(!Category::Mcps.is_supported_for(Editor::Codex, Scope::Project, root));
    }

    #[test]
    fn category_labels_match_install_values() {
        assert_eq!(Category::CursorPlugins.install_value(), "cursor-plugins");
        assert_eq!(Category::CodexSkills.install_value(), "codex-skills");
        assert_eq!(Category::CodexPlugins.install_value(), "codex-plugins");
    }

    #[test]
    fn cursor_plugin_hooks_enable_cursor_hooks_only() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("test clock should be valid")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("stacc-catalog-hooks-{unique}"));
        let hook_dir = root
            .join(CONFIGS_DIR)
            .join("cursor-plugins")
            .join("hooks")
            .join("continual-learning");
        fs::create_dir_all(&hook_dir).expect("hook dir should be created");

        assert!(Category::Hooks.is_supported_for(Editor::Cursor, Scope::Project, &root));
        assert!(!Category::Hooks.is_supported_for(Editor::Claude, Scope::Project, &root));

        let _ = fs::remove_dir_all(root);
    }
}
