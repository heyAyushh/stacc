use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::{default_panel_config_path, Category, ConflictMode, Editor, Scope};
use crate::install::InstallRequest;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PanelConfig {
    pub default_editors: Vec<Editor>,
    pub default_scope: Scope,
    pub default_categories: Vec<Category>,
    pub default_stacks: Vec<String>,
    pub default_mcp_servers: Vec<String>,
    pub default_hook_packages: Vec<String>,
    pub default_codex_plugins: Vec<String>,
    pub conflict_mode: ConflictMode,
    pub dry_run: bool,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            default_editors: vec![Editor::Cursor, Editor::Claude, Editor::Codex],
            default_scope: Scope::Project,
            default_categories: vec![
                Category::Rules,
                Category::Skills,
                Category::Stack,
                Category::Mcps,
            ],
            default_stacks: Vec::new(),
            default_mcp_servers: Vec::new(),
            default_hook_packages: Vec::new(),
            default_codex_plugins: Vec::new(),
            conflict_mode: ConflictMode::Backup,
            dry_run: true,
        }
    }
}

impl PanelConfig {
    pub fn from_install_request(request: &InstallRequest) -> Self {
        Self {
            default_editors: request.editors.clone(),
            default_scope: request.scope,
            default_categories: request.categories.clone(),
            default_stacks: request.stacks.clone(),
            default_mcp_servers: request.mcp_servers.clone(),
            default_hook_packages: request.hook_packages.clone(),
            default_codex_plugins: request.codex_plugins.clone(),
            conflict_mode: request.conflict_mode,
            dry_run: request.dry_run,
        }
    }
}

pub fn load_panel_config(root: &Path, config_path: Option<PathBuf>) -> Result<PanelConfig> {
    let path = config_path.unwrap_or_else(|| default_panel_config_path(root));
    if !path.is_file() {
        return Ok(PanelConfig::default());
    }

    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("invalid panel config {}", path.display()))
}
