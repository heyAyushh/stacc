use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use toml_edit::{value as toml_value, DocumentMut, Item as TomlItem, Table as TomlTable};

use crate::catalog::{Category, ConflictMode, Editor, Scope};
use crate::hook_selection;
use crate::selective::{prepare_existing_target_for_append, prepare_existing_target_for_write};

const AGENTS_FILE: &str = "AGENTS.md";
const BACKUP_EXTENSION_PREFIX: &str = "bak";
const CLAUDE_FILE: &str = "CLAUDE.md";
const CODEX_COMMAND: &str = "codex";
const CODEX_CONFIG_FILE: &str = "config.toml";
const CODEX_PLUGINS_CONFIG_FILE: &str = "plugins.json";
const CODEX_PLUGINS_KEY: &str = "plugins";
const CODEX_PLUGIN_INSTALL_EXAMPLE: &str =
    "stacc install --editor codex --codex-plugin lazycodex --dry-run --print-plan";
const COMMANDS_SKILLS_DIR: &str = "configs/commands/skills";
const CONFIGS_DIR: &str = "configs";
const CURSOR_RULES_DIR: &str = "rules";
const DEFAULT_PROJECT_CLAUDE_RULE_LIMIT: usize = 3;
const DOT_AGENT_DIR: &str = ".agents";
const DOT_CODEX_DIR: &str = ".codex";
const DOT_CURSOR_DIR: &str = ".cursor";
const DOT_CLAUDE_DIR: &str = ".claude";
const DOT_MCP_FILE: &str = ".mcp.json";
const DOT_OPENCODE_DIR: &str = ".opencode";
const DOT_OPENCODE_FILE: &str = ".opencode.json";
const GLOBAL_AGENT_RULES_DIR: &str = ".agents/rules";
const GLOBAL_AMP_DIR: &str = ".config/amp";
const GLOBAL_AMP_SKILLS_DIR: &str = ".config/agents/skills";
const GLOBAL_CLAUDE_DIR: &str = ".claude";
const GLOBAL_CLAUDE_MCP_FILE: &str = ".claude.json";
const GLOBAL_CODEX_DIR: &str = ".codex";
const GLOBAL_CURSOR_DIR: &str = ".cursor";
const GLOBAL_OPENCODE_DIR: &str = ".config/opencode";
const HOOKS_DIR: &str = "hooks";
const MCP_SERVERS_KEY: &str = "mcpServers";
const MCP_SERVERS_TOML_KEY: &str = "mcp_servers";
const METADATA_FILE_NAME: &str = ".DS_Store";
const MANAGED_SKILL_FILE_NAME: &str = "SKILL.md";
const MANIFEST_MANAGER: &str = "stacc";
const MANIFEST_SCHEMA_VERSION: u16 = 1;
const STACC_DIR: &str = ".stacc";
const STACC_MANIFEST_FILE: &str = "manifest.json";
const RULES_SUMMARY_FILE: &str = "configs/rules/summary.md";
const RULES_SUMMARY_MARKER: &str = "<!-- stacc:rules-summary -->";
const SKILLS_DIR: &str = "skills";
const STACKS_DIR: &str = "configs/stacks";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstallRequest {
    pub root: PathBuf,
    pub editors: Vec<Editor>,
    pub scope: Scope,
    pub categories: Vec<Category>,
    pub stacks: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub hook_packages: Vec<String>,
    pub codex_plugins: Vec<String>,
    pub conflict_mode: ConflictMode,
    pub yes: bool,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallPlan {
    pub editor: Editor,
    pub scope: Scope,
    pub target_root: PathBuf,
    pub operations: Vec<InstallOperation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum InstallOperation {
    CopyFile {
        source: PathBuf,
        destination: PathBuf,
    },
    BackupPath {
        path: PathBuf,
        backup: PathBuf,
    },
    RemovePath {
        path: PathBuf,
    },
    WriteFile {
        destination: PathBuf,
        contents: String,
    },
    AppendFile {
        destination: PathBuf,
        contents: String,
    },
    PromptConflict {
        target: PathBuf,
        choices: String,
    },
    RunCommand {
        program: String,
        args: Vec<String>,
    },
    UpdateManifest {
        path: PathBuf,
        upsert: Vec<ManagedEntry>,
        remove_ids: Vec<String>,
    },
    Skip {
        target: PathBuf,
        reason: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallRunResult {
    pub editor: Editor,
    pub operation_count: usize,
    pub target_root: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManageAction {
    Update,
    Uninstall,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManageRequest {
    pub root: PathBuf,
    pub editors: Vec<Editor>,
    pub scope: Scope,
    pub skills: Vec<String>,
    pub codex_plugins: Vec<String>,
    pub conflict_mode: ConflictMode,
    pub yes: bool,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncManifestRequest {
    pub root: PathBuf,
    pub editors: Vec<Editor>,
    pub scope: Scope,
    pub skills: Vec<String>,
    pub codex_plugins: Vec<String>,
    pub yes: bool,
    pub dry_run: bool,
}

#[derive(Clone, Debug)]
struct InstallContext {
    project_root: PathBuf,
    home: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct CodexPluginCatalog {
    plugins: BTreeMap<String, CodexPluginSpec>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
struct CodexPluginSpec {
    description: String,
    marketplace_source: String,
    marketplace: String,
    plugin: String,
    source_url: String,
    license: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SelectedCodexPlugin {
    name: String,
    spec: CodexPluginSpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SkillSourcePackage {
    category: Category,
    name: String,
    source: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ManagedManifest {
    schema_version: u16,
    manager: String,
    entries: Vec<ManagedEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ManagedEntry {
    pub id: String,
    pub kind: ManagedKind,
    pub name: String,
    pub editor: Editor,
    pub scope: Scope,
    pub category: Option<Category>,
    pub source: Option<String>,
    pub destination: Option<PathBuf>,
    pub selector: Option<String>,
    pub marketplace: Option<String>,
    pub marketplace_source: Option<String>,
    pub source_url: Option<String>,
    pub license: Option<String>,
    pub updated_unix_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManagedKind {
    Skill,
    CodexPlugin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectoryConflictMode {
    WholeDirectory,
    PerFile,
}

impl InstallRequest {
    pub fn validate(&self) -> Result<()> {
        if self.editors.is_empty() {
            anyhow::bail!("install needs at least one editor");
        }
        let has_codex_plugin_request = has_codex_plugin_request(self);
        if self.categories.is_empty() && !has_codex_plugin_request {
            anyhow::bail!("install needs at least one category or --codex-plugin");
        }
        if !self.dry_run && !self.yes {
            anyhow::bail!("review dry-run first, then pass --yes for writes");
        }
        if !self.root.join(CONFIGS_DIR).is_dir() {
            anyhow::bail!("configs/ missing under {}", self.root.display());
        }
        if has_codex_plugin_request && !self.editors.contains(&Editor::Codex) {
            anyhow::bail!(
                "codex-plugins requires --editor codex.\nExample: {CODEX_PLUGIN_INSTALL_EXAMPLE}"
            );
        }
        Ok(())
    }
}

impl ManageRequest {
    pub fn validate(&self, action: ManageAction) -> Result<()> {
        if self.editors.is_empty() {
            anyhow::bail!("{} needs at least one editor", action.command_name());
        }
        if self.skills.is_empty() && self.codex_plugins.is_empty() {
            anyhow::bail!(
                "{} needs at least one --skill or --codex-plugin",
                action.command_name()
            );
        }
        if !self.dry_run && !self.yes {
            anyhow::bail!("review dry-run first, then pass --yes for writes");
        }
        if !self.root.join(CONFIGS_DIR).is_dir() {
            anyhow::bail!("configs/ missing under {}", self.root.display());
        }
        if !self.codex_plugins.is_empty() && !self.editors.contains(&Editor::Codex) {
            anyhow::bail!(
                "{} --codex-plugin requires --editor codex",
                action.command_name()
            );
        }
        Ok(())
    }
}

impl SyncManifestRequest {
    pub fn validate(&self) -> Result<()> {
        if self.editors.is_empty() {
            anyhow::bail!("sync needs at least one editor");
        }
        if !self.dry_run && !self.yes {
            anyhow::bail!("review dry-run first, then pass --yes for writes");
        }
        if !self.root.join(CONFIGS_DIR).is_dir() {
            anyhow::bail!("configs/ missing under {}", self.root.display());
        }
        if !self.codex_plugins.is_empty() && !self.editors.contains(&Editor::Codex) {
            anyhow::bail!("sync --codex-plugin requires --editor codex");
        }
        Ok(())
    }
}

impl ManageAction {
    fn command_name(self) -> &'static str {
        match self {
            ManageAction::Update => "update",
            ManageAction::Uninstall => "uninstall",
        }
    }
}

pub fn build_install_plan(request: &InstallRequest) -> Result<Vec<InstallPlan>> {
    request.validate()?;
    let context = InstallContext::new()?;

    let mut plans = Vec::new();
    for editor in &request.editors {
        let categories = filtered_categories(request, *editor);
        if categories.is_empty() {
            continue;
        }

        let target_root = target_root_for(*editor, request.scope, &context)?;
        let mut plan = InstallPlan {
            editor: *editor,
            scope: request.scope,
            target_root,
            operations: Vec::new(),
        };
        install_for_target(request, &context, &mut plan, &categories)?;
        plans.push(plan);
    }
    if has_codex_plugin_request(request) {
        let target_root = target_root_for(Editor::Codex, Scope::Global, &context)?;
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Global,
            target_root,
            operations: Vec::new(),
        };
        install_codex_plugins(request, &mut plan)?;
        plans.push(plan);
    }

    if plans.is_empty() {
        anyhow::bail!("selected categories are unsupported for selected editors/scope");
    }

    Ok(plans)
}

pub fn build_manage_plan(
    request: &ManageRequest,
    action: ManageAction,
) -> Result<Vec<InstallPlan>> {
    request.validate(action)?;
    let context = InstallContext::new()?;
    let selected_skills = normalized_selected_names(&request.skills);
    let selected_plugins = normalized_selected_names(&request.codex_plugins);
    let mut plans = Vec::new();

    if !selected_skills.is_empty() {
        for editor in &request.editors {
            let target_root = target_root_for(*editor, request.scope, &context)?;
            let mut plan = InstallPlan {
                editor: *editor,
                scope: request.scope,
                target_root,
                operations: Vec::new(),
            };
            plan_managed_skills(request, &mut plan, &selected_skills, action)?;
            plans.push(plan);
        }
    }

    if !selected_plugins.is_empty() {
        let target_root = target_root_for(Editor::Codex, Scope::Global, &context)?;
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Global,
            target_root,
            operations: Vec::new(),
        };
        plan_managed_codex_plugins(request, &mut plan, &selected_plugins, action)?;
        plans.push(plan);
    }

    if plans.is_empty() {
        anyhow::bail!(
            "{} found no stacc-managed entries to plan",
            action.command_name()
        );
    }

    Ok(plans)
}

pub fn build_sync_manifest_plan(request: &SyncManifestRequest) -> Result<Vec<InstallPlan>> {
    request.validate()?;
    let context = InstallContext::new()?;
    let selected_skills = normalized_selected_names(&request.skills);
    let selected_plugins = normalized_selected_names(&request.codex_plugins);
    let mut plans = Vec::new();

    for editor in &request.editors {
        let target_root = target_root_for(*editor, request.scope, &context)?;
        let mut plan = InstallPlan {
            editor: *editor,
            scope: request.scope,
            target_root,
            operations: Vec::new(),
        };
        plan_manifest_skill_sync(request, &mut plan, &selected_skills)?;
        if !plan.operations.is_empty() || selected_plugins.is_empty() {
            plans.push(plan);
        }
    }

    if !selected_plugins.is_empty() {
        let target_root = target_root_for(Editor::Codex, Scope::Global, &context)?;
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Global,
            target_root,
            operations: Vec::new(),
        };
        plan_manifest_codex_plugin_sync(request, &mut plan, &selected_plugins)?;
        plans.push(plan);
    }

    if plans.is_empty() {
        anyhow::bail!("sync found no matching installed stacc entries");
    }

    Ok(plans)
}

pub fn execute_install_request(request: &InstallRequest) -> Result<Vec<InstallRunResult>> {
    let plans = build_install_plan(request)?;
    let mut results = Vec::with_capacity(plans.len());

    for plan in &plans {
        for operation in &plan.operations {
            execute_operation(operation, request)?;
        }
        results.push(InstallRunResult {
            editor: plan.editor,
            operation_count: plan.operations.len(),
            target_root: plan.target_root.clone(),
        });
    }

    Ok(results)
}

pub fn execute_manage_request(
    request: &ManageRequest,
    action: ManageAction,
) -> Result<Vec<InstallRunResult>> {
    let plans = build_manage_plan(request, action)?;
    let install_request = install_request_from_manage(request);
    let mut results = Vec::with_capacity(plans.len());

    for plan in &plans {
        for operation in &plan.operations {
            execute_operation(operation, &install_request)?;
        }
        results.push(InstallRunResult {
            editor: plan.editor,
            operation_count: plan.operations.len(),
            target_root: plan.target_root.clone(),
        });
    }

    Ok(results)
}

pub fn execute_sync_manifest_request(
    request: &SyncManifestRequest,
) -> Result<Vec<InstallRunResult>> {
    let plans = build_sync_manifest_plan(request)?;
    let install_request = install_request_from_sync_manifest(request);
    let mut results = Vec::with_capacity(plans.len());

    for plan in &plans {
        for operation in &plan.operations {
            execute_operation(operation, &install_request)?;
        }
        results.push(InstallRunResult {
            editor: plan.editor,
            operation_count: plan.operations.len(),
            target_root: plan.target_root.clone(),
        });
    }

    Ok(results)
}

pub fn print_plan(plans: &[InstallPlan]) {
    for plan in plans {
        println!(
            "{} {} -> {}",
            plan.editor,
            plan.scope,
            plan.target_root.display()
        );
        if plan.operations.is_empty() {
            println!("  no operations");
            continue;
        }
        for operation in &plan.operations {
            println!("  {}", operation.describe());
        }
    }
}

impl InstallContext {
    fn new() -> Result<Self> {
        Ok(Self {
            project_root: env::current_dir().context("failed to read current directory")?,
            home: home_dir()?,
        })
    }
}

impl InstallOperation {
    fn describe(&self) -> String {
        match self {
            InstallOperation::CopyFile {
                source,
                destination,
            } => {
                format!("copy {} -> {}", source.display(), destination.display())
            }
            InstallOperation::BackupPath { path, backup } => {
                format!("backup {} -> {}", path.display(), backup.display())
            }
            InstallOperation::RemovePath { path } => {
                format!("remove {}", path.display())
            }
            InstallOperation::WriteFile { destination, .. } => {
                format!("write {}", destination.display())
            }
            InstallOperation::AppendFile { destination, .. } => {
                format!("append {}", destination.display())
            }
            InstallOperation::PromptConflict { target, choices } => {
                format!("prompt {} ({choices})", target.display())
            }
            InstallOperation::RunCommand { program, args } => {
                format!("run {}", shell_words(program, args))
            }
            InstallOperation::UpdateManifest {
                path,
                upsert,
                remove_ids,
            } => {
                let mut parts = Vec::new();
                if !upsert.is_empty() {
                    parts.push(format!("upsert {}", upsert.len()));
                }
                if !remove_ids.is_empty() {
                    parts.push(format!("remove {}", remove_ids.len()));
                }
                format!("update manifest {} ({})", path.display(), parts.join(", "))
            }
            InstallOperation::Skip { target, reason } => {
                format!("skip {} ({reason})", target.display())
            }
        }
    }
}

fn install_for_target(
    request: &InstallRequest,
    context: &InstallContext,
    plan: &mut InstallPlan,
    categories: &[Category],
) -> Result<()> {
    let mut selected_rules = false;
    let mut selected_skills = false;

    for category in categories {
        match category {
            Category::Mcps => install_mcp(request, context, plan)?,
            Category::Rules => {
                selected_rules = true;
                install_rules(request, context, plan)?;
            }
            Category::Skills => {
                selected_skills = true;
                install_skills(request, plan)?;
            }
            Category::CursorPlugins => {
                selected_skills = true;
                install_cursor_plugins(request, plan)?;
            }
            Category::CodexSkills => {
                selected_skills = true;
                install_codex_skills(request, plan)?;
            }
            Category::CodexPlugins => install_codex_plugins(request, plan)?,
            Category::Commands => install_commands(request, plan)?,
            Category::Stack => install_stacks(request, plan)?,
            Category::Hooks => install_hooks(request, plan)?,
            Category::Agents => {
                install_category(request, plan, *category, category.install_value())?;
            }
        }
    }

    if selected_skills && !selected_rules {
        install_rules(request, context, plan)?;
    }

    Ok(())
}

fn filtered_categories(request: &InstallRequest, editor: Editor) -> Vec<Category> {
    request
        .categories
        .iter()
        .copied()
        .filter(|category| *category != Category::CodexPlugins)
        .filter(|category| category.is_supported_for(editor, request.scope, &request.root))
        .collect()
}

fn has_codex_plugin_request(request: &InstallRequest) -> bool {
    !request.codex_plugins.is_empty() || request.categories.contains(&Category::CodexPlugins)
}

fn manifest_path_for(target_root: &Path) -> PathBuf {
    target_root.join(STACC_DIR).join(STACC_MANIFEST_FILE)
}

fn plan_manifest_upsert(plan: &mut InstallPlan, entries: Vec<ManagedEntry>) {
    if entries.is_empty() {
        return;
    }
    plan.operations.push(InstallOperation::UpdateManifest {
        path: manifest_path_for(&plan.target_root),
        upsert: entries,
        remove_ids: Vec::new(),
    });
}

fn plan_manifest_remove(plan: &mut InstallPlan, remove_ids: Vec<String>) {
    if remove_ids.is_empty() {
        return;
    }
    plan.operations.push(InstallOperation::UpdateManifest {
        path: manifest_path_for(&plan.target_root),
        upsert: Vec::new(),
        remove_ids,
    });
}

fn managed_timestamp_for_dry_run(dry_run: bool) -> u64 {
    if dry_run {
        0
    } else {
        unix_seconds_now()
    }
}

fn unix_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn managed_entry_id(kind: ManagedKind, editor: Editor, scope: Scope, name: &str) -> String {
    format!("{}:{}:{}:{name}", editor, scope, kind.install_value())
}

impl ManagedKind {
    fn install_value(self) -> &'static str {
        match self {
            ManagedKind::Skill => "skill",
            ManagedKind::CodexPlugin => "codex-plugin",
        }
    }
}

fn install_request_from_manage(request: &ManageRequest) -> InstallRequest {
    InstallRequest {
        root: request.root.clone(),
        editors: request.editors.clone(),
        scope: request.scope,
        categories: Vec::new(),
        stacks: Vec::new(),
        mcp_servers: Vec::new(),
        hook_packages: Vec::new(),
        codex_plugins: request.codex_plugins.clone(),
        conflict_mode: request.conflict_mode,
        yes: request.yes,
        dry_run: request.dry_run,
    }
}

fn install_request_from_sync_manifest(request: &SyncManifestRequest) -> InstallRequest {
    InstallRequest {
        root: request.root.clone(),
        editors: request.editors.clone(),
        scope: request.scope,
        categories: Vec::new(),
        stacks: Vec::new(),
        mcp_servers: Vec::new(),
        hook_packages: Vec::new(),
        codex_plugins: request.codex_plugins.clone(),
        conflict_mode: ConflictMode::Backup,
        yes: request.yes,
        dry_run: request.dry_run,
    }
}

fn plan_manifest_skill_sync(
    request: &SyncManifestRequest,
    plan: &mut InstallPlan,
    selected_skills: &[String],
) -> Result<()> {
    let sources = syncable_skill_sources(&request.root, plan.editor)?;
    let destination_root = skills_root_for(plan.editor, plan.scope, &plan.target_root)?;
    let install_request = install_request_from_sync_manifest(request);
    let mut upsert = Vec::new();
    let mut missing_selected = selected_skills.iter().cloned().collect::<BTreeSet<_>>();

    for source in sources {
        if !selected_skills.is_empty() && !selected_skills.contains(&source.name) {
            continue;
        }
        let destination = destination_root.join(&source.name);
        if installed_skill_package_exists(&destination) {
            upsert.push(skill_manifest_entry(
                &install_request,
                plan,
                source.category,
                &source.name,
                &source.source,
                &destination,
            )?);
            missing_selected.remove(&source.name);
        }
    }

    for skill in missing_selected {
        plan.operations.push(InstallOperation::Skip {
            target: destination_root.join(&skill),
            reason: "installed stacc skill not found".to_string(),
        });
    }
    plan_manifest_upsert(plan, upsert);
    Ok(())
}

fn plan_manifest_codex_plugin_sync(
    request: &SyncManifestRequest,
    plan: &mut InstallPlan,
    selected_plugins: &[String],
) -> Result<()> {
    if selected_plugins.is_empty() {
        return Ok(());
    }
    if plan.editor != Editor::Codex || plan.scope != Scope::Global {
        anyhow::bail!("codex plugin manifest sync is only supported for global Codex installs");
    }

    let install_request = install_request_from_sync_manifest(request);
    let catalog = read_codex_plugin_catalog(&install_request)?;
    let available = available_codex_plugin_names(&catalog.plugins);
    let unknown = unknown_names(selected_plugins, &available);
    if !unknown.is_empty() {
        anyhow::bail!(
            "unknown Codex plugin(s): {}. Available: {}",
            unknown.join(","),
            available.join(",")
        );
    }

    let mut entries = Vec::with_capacity(selected_plugins.len());
    for plugin in selected_plugins {
        let spec = catalog
            .plugins
            .get(plugin)
            .cloned()
            .with_context(|| format!("Codex plugin disappeared while syncing {plugin}"))?;
        entries.push(codex_plugin_manifest_entry_from_parts(
            request.dry_run,
            &SelectedCodexPlugin {
                name: plugin.clone(),
                spec,
            },
        ));
    }
    plan_manifest_upsert(plan, entries);
    Ok(())
}

fn syncable_skill_sources(root: &Path, editor: Editor) -> Result<Vec<SkillSourcePackage>> {
    let mut sources = Vec::new();
    append_skill_source_packages(
        &mut sources,
        Category::Skills,
        &root
            .join(CONFIGS_DIR)
            .join(Category::Skills.install_value()),
    )?;
    append_stack_source_packages(&mut sources, root)?;

    if shares_skills_root(editor) {
        append_skill_source_packages(
            &mut sources,
            Category::Commands,
            &root.join(COMMANDS_SKILLS_DIR),
        )?;
    }
    if editor == Editor::Cursor {
        append_skill_source_packages(
            &mut sources,
            Category::CursorPlugins,
            &root
                .join(CONFIGS_DIR)
                .join(Category::CursorPlugins.install_value())
                .join(SKILLS_DIR),
        )?;
    }
    if editor == Editor::Codex {
        append_skill_source_packages(
            &mut sources,
            Category::CodexSkills,
            &root
                .join(CONFIGS_DIR)
                .join(Category::CodexSkills.install_value())
                .join(SKILLS_DIR),
        )?;
    }

    sources.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.category.cmp(&right.category))
    });
    Ok(sources)
}

fn append_skill_source_packages(
    sources: &mut Vec<SkillSourcePackage>,
    category: Category,
    source_root: &Path,
) -> Result<()> {
    if !source_root.is_dir() {
        return Ok(());
    }
    for name in skill_package_names(source_root)? {
        sources.push(SkillSourcePackage {
            category,
            source: source_root.join(&name),
            name,
        });
    }
    Ok(())
}

fn append_stack_source_packages(sources: &mut Vec<SkillSourcePackage>, root: &Path) -> Result<()> {
    let stack_root = root.join(STACKS_DIR);
    if !stack_root.is_dir() {
        return Ok(());
    }
    for stack in available_stack_names(root)? {
        sources.push(SkillSourcePackage {
            category: Category::Stack,
            source: stack_root.join(&stack),
            name: stack,
        });
    }
    Ok(())
}

fn installed_skill_package_exists(destination: &Path) -> bool {
    destination.join(MANAGED_SKILL_FILE_NAME).is_file()
}

fn plan_managed_skills(
    request: &ManageRequest,
    plan: &mut InstallPlan,
    selected_skills: &[String],
    action: ManageAction,
) -> Result<()> {
    let manifest_path = manifest_path_for(&plan.target_root);
    let manifest = read_required_manifest(&manifest_path)?;
    let install_request = install_request_from_manage(request);
    let mut upsert = Vec::new();
    let mut remove_ids = Vec::new();

    for skill in selected_skills {
        let entry = find_managed_entry(
            &manifest,
            ManagedKind::Skill,
            plan.editor,
            plan.scope,
            skill,
        )
        .with_context(|| {
            format!(
                "no stacc-managed skill `{skill}` for {} {} in {}",
                plan.editor,
                plan.scope,
                manifest_path.display()
            )
        })?
        .clone();
        let destination = validated_managed_skill_destination(&entry, &plan.target_root)?;

        match action {
            ManageAction::Update => {
                let source = managed_entry_source_path(request, &entry)?;
                copy_tree(
                    &source,
                    &destination,
                    None,
                    DirectoryConflictMode::WholeDirectory,
                    &install_request,
                    plan,
                )?;
                let mut updated = entry;
                updated.updated_unix_seconds = managed_timestamp_for_dry_run(request.dry_run);
                upsert.push(updated);
            }
            ManageAction::Uninstall => {
                if destination.exists() {
                    plan.operations
                        .push(InstallOperation::RemovePath { path: destination });
                } else {
                    plan.operations.push(InstallOperation::Skip {
                        target: destination,
                        reason: "managed skill is already absent".to_string(),
                    });
                }
                remove_ids.push(entry.id);
            }
        }
    }

    match action {
        ManageAction::Update => plan_manifest_upsert(plan, upsert),
        ManageAction::Uninstall => plan_manifest_remove(plan, remove_ids),
    }
    Ok(())
}

fn plan_managed_codex_plugins(
    request: &ManageRequest,
    plan: &mut InstallPlan,
    selected_plugins: &[String],
    action: ManageAction,
) -> Result<()> {
    if plan.editor != Editor::Codex || plan.scope != Scope::Global {
        anyhow::bail!("codex plugin management is only supported for global Codex installs");
    }

    let manifest_path = manifest_path_for(&plan.target_root);
    let manifest = read_required_manifest(&manifest_path)?;
    let install_request = install_request_from_manage(request);
    let catalog = if action == ManageAction::Update {
        Some(read_codex_plugin_catalog(&install_request)?)
    } else {
        None
    };
    let mut upsert = Vec::new();
    let mut remove_ids = Vec::new();
    let mut uninstall_marketplaces = BTreeSet::new();

    for plugin in selected_plugins {
        let entry = find_managed_entry(
            &manifest,
            ManagedKind::CodexPlugin,
            Editor::Codex,
            Scope::Global,
            plugin,
        )
        .with_context(|| {
            format!(
                "no stacc-managed Codex plugin `{plugin}` in {}",
                manifest_path.display()
            )
        })?
        .clone();

        match action {
            ManageAction::Update => {
                let catalog = catalog
                    .as_ref()
                    .context("Codex plugin catalog missing during update")?;
                let spec = catalog.plugins.get(plugin).with_context(|| {
                    format!("Codex plugin `{plugin}` is no longer present in the stacc catalog")
                })?;
                let selected = SelectedCodexPlugin {
                    name: plugin.clone(),
                    spec: spec.clone(),
                };
                plan.operations.push(InstallOperation::RunCommand {
                    program: CODEX_COMMAND.to_string(),
                    args: vec![
                        "plugin".to_string(),
                        "marketplace".to_string(),
                        "add".to_string(),
                        codex_cli_token(&selected.spec.marketplace_source, "marketplace source")?,
                    ],
                });
                plan.operations.push(InstallOperation::RunCommand {
                    program: CODEX_COMMAND.to_string(),
                    args: vec![
                        "plugin".to_string(),
                        "marketplace".to_string(),
                        "upgrade".to_string(),
                        codex_cli_token(&selected.spec.marketplace, "marketplace name")?,
                    ],
                });
                plan.operations.push(InstallOperation::RunCommand {
                    program: CODEX_COMMAND.to_string(),
                    args: vec![
                        "plugin".to_string(),
                        "add".to_string(),
                        codex_cli_token(&selected.spec.plugin, "plugin selector")?,
                    ],
                });
                upsert.push(codex_plugin_manifest_entry_from_parts(
                    request.dry_run,
                    &selected,
                ));
            }
            ManageAction::Uninstall => {
                let selector = entry
                    .selector
                    .as_deref()
                    .context("managed Codex plugin entry missing selector")?;
                plan.operations.push(InstallOperation::RunCommand {
                    program: CODEX_COMMAND.to_string(),
                    args: vec![
                        "plugin".to_string(),
                        "remove".to_string(),
                        codex_cli_token(selector, "plugin selector")?,
                    ],
                });
                if let Some(marketplace) = entry.marketplace.as_deref() {
                    uninstall_marketplaces
                        .insert(codex_cli_token(marketplace, "marketplace name")?);
                }
                remove_ids.push(entry.id);
            }
        }
    }

    if action == ManageAction::Uninstall {
        for marketplace in uninstall_marketplaces {
            if !manifest.entries.iter().any(|entry| {
                !remove_ids.contains(&entry.id)
                    && entry.kind == ManagedKind::CodexPlugin
                    && entry.marketplace.as_deref() == Some(marketplace.as_str())
            }) {
                plan.operations.push(InstallOperation::RunCommand {
                    program: CODEX_COMMAND.to_string(),
                    args: vec![
                        "plugin".to_string(),
                        "marketplace".to_string(),
                        "remove".to_string(),
                        marketplace,
                    ],
                });
            }
        }
    }

    match action {
        ManageAction::Update => plan_manifest_upsert(plan, upsert),
        ManageAction::Uninstall => plan_manifest_remove(plan, remove_ids),
    }
    Ok(())
}

fn find_managed_entry<'manifest>(
    manifest: &'manifest ManagedManifest,
    kind: ManagedKind,
    editor: Editor,
    scope: Scope,
    name: &str,
) -> Option<&'manifest ManagedEntry> {
    manifest.entries.iter().find(|entry| {
        entry.kind == kind && entry.editor == editor && entry.scope == scope && entry.name == name
    })
}

fn install_skill_packages(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    category: Category,
    source_root: &Path,
    destination_root: &Path,
    directory_conflict_mode: DirectoryConflictMode,
) -> Result<()> {
    let package_names = skill_package_names(source_root)?;
    let mut manifest_entries = Vec::with_capacity(package_names.len());
    for package in package_names {
        let source = source_root.join(&package);
        let destination = destination_root.join(&package);
        copy_tree(
            &source,
            &destination,
            None,
            directory_conflict_mode,
            request,
            plan,
        )?;
        manifest_entries.push(skill_manifest_entry(
            request,
            plan,
            category,
            &package,
            &source,
            &destination,
        )?);
    }
    plan_manifest_upsert(plan, manifest_entries);
    Ok(())
}

fn skill_package_names(source_root: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(source_root)
        .with_context(|| format!("failed to read {}", source_root.display()))?
    {
        let entry = entry
            .with_context(|| format!("failed to read entry under {}", source_root.display()))?;
        let path = entry.path();
        if entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", path.display()))?
            .is_dir()
            && path.join(MANAGED_SKILL_FILE_NAME).is_file()
        {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

fn skill_manifest_entry(
    request: &InstallRequest,
    plan: &InstallPlan,
    category: Category,
    name: &str,
    source: &Path,
    destination: &Path,
) -> Result<ManagedEntry> {
    Ok(ManagedEntry {
        id: managed_entry_id(ManagedKind::Skill, plan.editor, plan.scope, name),
        kind: ManagedKind::Skill,
        name: name.to_string(),
        editor: plan.editor,
        scope: plan.scope,
        category: Some(category),
        source: Some(managed_source_string(&request.root, source)?),
        destination: Some(managed_destination_path(&plan.target_root, destination)?),
        selector: None,
        marketplace: None,
        marketplace_source: None,
        source_url: None,
        license: None,
        updated_unix_seconds: managed_timestamp_for_dry_run(request.dry_run),
    })
}

fn codex_plugin_manifest_entry(
    request: &InstallRequest,
    plugin: &SelectedCodexPlugin,
) -> ManagedEntry {
    codex_plugin_manifest_entry_from_parts(request.dry_run, plugin)
}

fn codex_plugin_manifest_entry_from_parts(
    dry_run: bool,
    plugin: &SelectedCodexPlugin,
) -> ManagedEntry {
    ManagedEntry {
        id: managed_entry_id(
            ManagedKind::CodexPlugin,
            Editor::Codex,
            Scope::Global,
            &plugin.name,
        ),
        kind: ManagedKind::CodexPlugin,
        name: plugin.name.clone(),
        editor: Editor::Codex,
        scope: Scope::Global,
        category: Some(Category::CodexPlugins),
        source: None,
        destination: None,
        selector: Some(plugin.spec.plugin.clone()),
        marketplace: Some(plugin.spec.marketplace.clone()),
        marketplace_source: Some(plugin.spec.marketplace_source.clone()),
        source_url: Some(plugin.spec.source_url.clone()),
        license: Some(plugin.spec.license.clone()),
        updated_unix_seconds: managed_timestamp_for_dry_run(dry_run),
    }
}

fn managed_source_string(root: &Path, source: &Path) -> Result<String> {
    let relative = source.strip_prefix(root).with_context(|| {
        format!(
            "managed source {} is not under {}",
            source.display(),
            root.display()
        )
    })?;
    validate_relative_path(relative)?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn managed_destination_path(target_root: &Path, destination: &Path) -> Result<PathBuf> {
    if let Ok(relative) = destination.strip_prefix(target_root) {
        if !relative.as_os_str().is_empty() {
            return validate_relative_path(relative);
        }
    }
    normalize_absolute_path(destination)
}

fn managed_entry_source_path(request: &ManageRequest, entry: &ManagedEntry) -> Result<PathBuf> {
    let source = entry
        .source
        .as_deref()
        .context("managed skill entry missing source")?;
    let relative = validate_relative_path(Path::new(source))?;
    let source = request.root.join(relative);
    if !source.is_dir() {
        anyhow::bail!("managed source directory missing: {}", source.display());
    }
    Ok(source)
}

fn validate_relative_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        anyhow::bail!(
            "managed relative path must not be absolute: {}",
            path.display()
        );
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("managed relative path escapes its root: {}", path.display());
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        anyhow::bail!("managed relative path must not be empty");
    }
    Ok(normalized)
}

fn validated_managed_skill_destination(
    entry: &ManagedEntry,
    target_root: &Path,
) -> Result<PathBuf> {
    let destination = entry
        .destination
        .as_deref()
        .context("managed skill entry missing destination")?;
    let destination = if destination.is_absolute() {
        normalize_absolute_path(destination)?
    } else {
        let relative = validate_relative_path(destination)?;
        normalize_absolute_path(&target_root.join(relative))?
    };
    let allowed_root = skills_root_for(entry.editor, entry.scope, target_root)?;
    let allowed_root = normalize_absolute_path(&allowed_root)?;
    if !destination.starts_with(&allowed_root) {
        anyhow::bail!(
            "managed destination {} is outside {}",
            destination.display(),
            allowed_root.display()
        );
    }
    Ok(destination)
}

fn normalize_absolute_path(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        anyhow::bail!("managed path must be absolute: {}", path.display());
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    anyhow::bail!("managed path escapes filesystem root: {}", path.display());
                }
            }
        }
    }
    Ok(normalized)
}

fn codex_cli_token(value: &str, label: &str) -> Result<String> {
    let token = value.trim();
    if token.is_empty() || token != value {
        anyhow::bail!("invalid {label}: expected a non-empty token without surrounding whitespace");
    }
    if token.chars().any(char::is_whitespace) {
        anyhow::bail!("invalid {label}: whitespace is not supported");
    }
    Ok(token.to_string())
}

fn empty_manifest() -> ManagedManifest {
    ManagedManifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        manager: MANIFEST_MANAGER.to_string(),
        entries: Vec::new(),
    }
}

fn read_manifest(path: &Path) -> Result<ManagedManifest> {
    if !path.is_file() {
        return Ok(empty_manifest());
    }
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let manifest: ManagedManifest = serde_json::from_str(&contents)
        .with_context(|| format!("invalid stacc manifest {}", path.display()))?;
    validate_manifest(path, manifest)
}

fn read_required_manifest(path: &Path) -> Result<ManagedManifest> {
    if !path.is_file() {
        anyhow::bail!(
            "stacc manifest missing at {}; install the skill or plugin with stacc first",
            path.display()
        );
    }
    read_manifest(path)
}

fn validate_manifest(path: &Path, manifest: ManagedManifest) -> Result<ManagedManifest> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported stacc manifest schema {} in {}",
            manifest.schema_version,
            path.display()
        );
    }
    if manifest.manager != MANIFEST_MANAGER {
        anyhow::bail!(
            "manifest {} is managed by {}, not stacc",
            path.display(),
            manifest.manager
        );
    }
    Ok(manifest)
}

fn apply_manifest_update(
    path: &Path,
    upsert: &[ManagedEntry],
    remove_ids: &[String],
) -> Result<()> {
    let mut manifest = read_manifest(path)?;
    let upsert_ids = upsert
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();
    let remove_ids = remove_ids.iter().cloned().collect::<BTreeSet<_>>();
    manifest
        .entries
        .retain(|entry| !remove_ids.contains(&entry.id) && !upsert_ids.contains(&entry.id));
    manifest.entries.extend(upsert.iter().cloned());
    manifest
        .entries
        .sort_by(|left, right| left.id.cmp(&right.id));
    ensure_parent(path)?;
    let mut contents =
        serde_json::to_string_pretty(&manifest).context("failed to serialize stacc manifest")?;
    contents.push('\n');
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

fn install_category(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    category: Category,
    destination_subdir: &str,
) -> Result<()> {
    let source = request
        .root
        .join(CONFIGS_DIR)
        .join(category.install_value());
    let destination = plan.target_root.join(destination_subdir);
    copy_tree(
        &source,
        &destination,
        None,
        DirectoryConflictMode::WholeDirectory,
        request,
        plan,
    )
}

fn install_rules(
    request: &InstallRequest,
    context: &InstallContext,
    plan: &mut InstallPlan,
) -> Result<()> {
    let source = request
        .root
        .join(CONFIGS_DIR)
        .join(Category::Rules.install_value());
    let summary = request.root.join(RULES_SUMMARY_FILE);
    let rules_root = rules_root_for(plan.editor, plan.scope, &plan.target_root, context);
    copy_tree(
        &source,
        &rules_root,
        Some(summary.as_path()),
        DirectoryConflictMode::WholeDirectory,
        request,
        plan,
    )?;

    if plan.editor != Editor::Cursor {
        let destination =
            rules_summary_target_for(plan.editor, plan.scope, &plan.target_root, context);
        append_rules_summary(&summary, &destination, request, plan)?;
    }
    Ok(())
}

fn install_skills(request: &InstallRequest, plan: &mut InstallPlan) -> Result<()> {
    let source = request
        .root
        .join(CONFIGS_DIR)
        .join(Category::Skills.install_value());
    let destination = skills_root_for(plan.editor, plan.scope, &plan.target_root)?;
    let mode = if shares_skills_root(plan.editor) {
        DirectoryConflictMode::PerFile
    } else {
        DirectoryConflictMode::WholeDirectory
    };
    install_skill_packages(request, plan, Category::Skills, &source, &destination, mode)
}

fn install_cursor_plugins(request: &InstallRequest, plan: &mut InstallPlan) -> Result<()> {
    if plan.editor != Editor::Cursor {
        anyhow::bail!("cursor-plugins category is only supported for Cursor");
    }
    let source = request
        .root
        .join(CONFIGS_DIR)
        .join(Category::CursorPlugins.install_value());
    let skills = source.join(SKILLS_DIR);
    if skills.is_dir() {
        let destination = skills_root_for(plan.editor, plan.scope, &plan.target_root)?;
        install_skill_packages(
            request,
            plan,
            Category::CursorPlugins,
            &skills,
            &destination,
            DirectoryConflictMode::PerFile,
        )?;
    }

    for category in [Category::Agents, Category::Hooks] {
        let source = source.join(category.install_value());
        if source.is_dir() {
            let destination = plan.target_root.join(category.install_value());
            copy_tree(
                &source,
                &destination,
                None,
                DirectoryConflictMode::PerFile,
                request,
                plan,
            )?;
        }
    }
    Ok(())
}

fn install_codex_skills(request: &InstallRequest, plan: &mut InstallPlan) -> Result<()> {
    if plan.editor != Editor::Codex {
        anyhow::bail!("codex-skills category is only supported for Codex");
    }
    let source = request
        .root
        .join(CONFIGS_DIR)
        .join(Category::CodexSkills.install_value())
        .join(SKILLS_DIR);
    if source.is_dir() {
        let destination = skills_root_for(plan.editor, plan.scope, &plan.target_root)?;
        install_skill_packages(
            request,
            plan,
            Category::CodexSkills,
            &source,
            &destination,
            DirectoryConflictMode::PerFile,
        )?;
    }
    Ok(())
}

fn install_codex_plugins(request: &InstallRequest, plan: &mut InstallPlan) -> Result<()> {
    if plan.editor != Editor::Codex || plan.scope != Scope::Global {
        anyhow::bail!("codex-plugins category is only supported for global Codex installs");
    }

    let selected_plugins = selected_codex_plugins(request)?;
    let mut manifest_entries = Vec::with_capacity(selected_plugins.len());
    for plugin in selected_plugins {
        plan.operations.push(InstallOperation::RunCommand {
            program: CODEX_COMMAND.to_string(),
            args: vec![
                "plugin".to_string(),
                "marketplace".to_string(),
                "add".to_string(),
                codex_cli_token(&plugin.spec.marketplace_source, "marketplace source")?,
            ],
        });
        plan.operations.push(InstallOperation::RunCommand {
            program: CODEX_COMMAND.to_string(),
            args: vec![
                "plugin".to_string(),
                "add".to_string(),
                codex_cli_token(&plugin.spec.plugin, "plugin selector")?,
            ],
        });
        manifest_entries.push(codex_plugin_manifest_entry(request, &plugin));
    }
    plan_manifest_upsert(plan, manifest_entries);

    Ok(())
}

fn selected_codex_plugins(request: &InstallRequest) -> Result<Vec<SelectedCodexPlugin>> {
    let catalog = read_codex_plugin_catalog(request)?;
    let selected_plugins = normalized_selected_names(&request.codex_plugins);
    let available = available_codex_plugin_names(&catalog.plugins);
    if selected_plugins.is_empty() {
        anyhow::bail!(
            "codex-plugins requires --codex-plugin.\nExample: {CODEX_PLUGIN_INSTALL_EXAMPLE}\nAvailable: {}",
            available.join(",")
        );
    }
    let unknown = unknown_names(&selected_plugins, &available);
    if !unknown.is_empty() {
        anyhow::bail!(
            "unknown Codex plugin(s): {}.\nExample: {CODEX_PLUGIN_INSTALL_EXAMPLE}\nAvailable: {}",
            unknown.join(","),
            available.join(",")
        );
    }

    selected_plugins
        .into_iter()
        .map(|name| {
            let spec = catalog
                .plugins
                .get(&name)
                .cloned()
                .with_context(|| format!("Codex plugin disappeared while selecting {name}"))?;
            Ok(SelectedCodexPlugin { name, spec })
        })
        .collect()
}

fn read_codex_plugin_catalog(request: &InstallRequest) -> Result<CodexPluginCatalog> {
    let path = request
        .root
        .join(CONFIGS_DIR)
        .join(Category::CodexPlugins.install_value())
        .join(CODEX_PLUGINS_CONFIG_FILE);
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let catalog: CodexPluginCatalog = serde_json::from_str(&contents)
        .with_context(|| format!("invalid Codex plugin JSON {}", path.display()))?;
    if catalog.plugins.is_empty() {
        anyhow::bail!("Codex plugin catalog missing {CODEX_PLUGINS_KEY} entries");
    }
    Ok(catalog)
}

fn available_codex_plugin_names(plugins: &BTreeMap<String, CodexPluginSpec>) -> Vec<String> {
    plugins.keys().cloned().collect()
}

fn install_commands(request: &InstallRequest, plan: &mut InstallPlan) -> Result<()> {
    let (source, destination, exclude, mode) = if shares_skills_root(plan.editor) {
        (
            request.root.join(COMMANDS_SKILLS_DIR),
            skills_root_for(plan.editor, plan.scope, &plan.target_root)?,
            None,
            DirectoryConflictMode::PerFile,
        )
    } else {
        let source = request
            .root
            .join(CONFIGS_DIR)
            .join(Category::Commands.install_value());
        let exclude = source.join(SKILLS_DIR);
        (
            source,
            plan.target_root.join(Category::Commands.install_value()),
            Some(exclude),
            DirectoryConflictMode::WholeDirectory,
        )
    };

    if shares_skills_root(plan.editor) {
        return install_skill_packages(
            request,
            plan,
            Category::Commands,
            &source,
            &destination,
            mode,
        );
    }

    copy_tree(
        &source,
        &destination,
        exclude.as_deref(),
        mode,
        request,
        plan,
    )
}

fn install_stacks(request: &InstallRequest, plan: &mut InstallPlan) -> Result<()> {
    let selected_stacks = selected_stack_names(request)?;
    if selected_stacks.is_empty() {
        return Ok(());
    }

    let destination_root = skills_root_for(plan.editor, plan.scope, &plan.target_root)?;
    let mut manifest_entries = Vec::with_capacity(selected_stacks.len());
    for stack in selected_stacks {
        let source = request.root.join(STACKS_DIR).join(&stack);
        let destination = destination_root.join(&stack);
        copy_tree(
            &source,
            &destination,
            None,
            DirectoryConflictMode::WholeDirectory,
            request,
            plan,
        )?;
        manifest_entries.push(skill_manifest_entry(
            request,
            plan,
            Category::Stack,
            &stack,
            &source,
            &destination,
        )?);
    }
    plan_manifest_upsert(plan, manifest_entries);
    Ok(())
}

fn install_hooks(request: &InstallRequest, plan: &mut InstallPlan) -> Result<()> {
    let selected_hooks =
        hook_selection::selected_hook_packages(&request.root, plan.editor, &request.hook_packages)?;
    if selected_hooks.is_empty() {
        return Ok(());
    }

    let destination_root = plan.target_root.join(HOOKS_DIR);
    for (name, source) in selected_hooks {
        copy_tree(
            &source,
            &destination_root.join(name),
            None,
            DirectoryConflictMode::PerFile,
            request,
            plan,
        )?;
    }
    Ok(())
}

fn selected_stack_names(request: &InstallRequest) -> Result<Vec<String>> {
    let selected = normalized_selected_names(&request.stacks);
    if selected.is_empty() {
        return Ok(selected);
    }

    let available = available_stack_names(&request.root)?;
    if selected.iter().any(|stack| stack == "all") {
        if selected.len() > 1 {
            anyhow::bail!("--stack all cannot be combined with specific stack names");
        }
        return Ok(available);
    }

    let unknown = unknown_names(&selected, &available);
    if !unknown.is_empty() {
        anyhow::bail!(
            "unknown stack(s): {}. Available: {}",
            unknown.join(","),
            available.join(",")
        );
    }

    Ok(selected)
}

fn available_stack_names(root: &Path) -> Result<Vec<String>> {
    let stack_root = root.join(STACKS_DIR);
    let mut stacks = Vec::new();
    for entry in fs::read_dir(&stack_root)
        .with_context(|| format!("failed to read {}", stack_root.display()))?
    {
        let entry = entry
            .with_context(|| format!("failed to read entry under {}", stack_root.display()))?;
        if entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", entry.path().display()))?
            .is_dir()
        {
            stacks.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    stacks.sort();
    Ok(stacks)
}

fn install_mcp(
    request: &InstallRequest,
    context: &InstallContext,
    plan: &mut InstallPlan,
) -> Result<()> {
    let source = selected_mcp_config(request)?;
    let destination = mcp_path_for(plan.editor, plan.scope, &plan.target_root, context);
    let contents = match plan.editor {
        Editor::Codex => build_codex_mcp_config(&source, &destination, request)?,
        Editor::Ampcode => {
            let mut amp = Map::new();
            amp.insert("amp".to_string(), source);
            format_json(&Value::Object(amp))?
        }
        Editor::Cursor | Editor::Claude | Editor::Opencode => {
            let merged = merged_json_target(&source, &destination)?;
            format_json(&merged)?
        }
    };

    plan_write_file(request, plan, destination, contents)
}

fn selected_mcp_config(request: &InstallRequest) -> Result<Value> {
    let path = request.root.join(CONFIGS_DIR).join("mcps").join("mcp.json");
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let source: Value = serde_json::from_str(&contents)
        .with_context(|| format!("invalid MCP JSON {}", path.display()))?;
    let selected_servers = normalized_selected_names(&request.mcp_servers);
    if selected_servers.is_empty() {
        return Ok(source);
    }

    let servers = source
        .get(MCP_SERVERS_KEY)
        .and_then(Value::as_object)
        .context("MCP config missing mcpServers object")?;
    let available = available_mcp_server_names(servers);
    let unknown = unknown_names(&selected_servers, &available);
    if !unknown.is_empty() {
        anyhow::bail!(
            "unknown MCP server(s): {}. Available: {}",
            unknown.join(","),
            available.join(",")
        );
    }

    let mut selected = Map::new();
    for key in selected_servers {
        let server = servers
            .get(&key)
            .with_context(|| format!("MCP server disappeared while selecting {key}"))?;
        selected.insert(key, server.clone());
    }

    let mut root = Map::new();
    root.insert(MCP_SERVERS_KEY.to_string(), Value::Object(selected));
    Ok(Value::Object(root))
}

fn normalized_selected_names(values: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    for value in values {
        let name = value.trim();
        if name.is_empty() || names.iter().any(|existing| existing == name) {
            continue;
        }
        names.push(name.to_string());
    }
    names
}

fn unknown_names(selected: &[String], available: &[String]) -> Vec<String> {
    selected
        .iter()
        .filter(|name| {
            !available
                .iter()
                .any(|available_name| available_name == *name)
        })
        .cloned()
        .collect()
}

fn available_mcp_server_names(servers: &Map<String, Value>) -> Vec<String> {
    let mut names = servers.keys().cloned().collect::<Vec<_>>();
    names.sort();
    names
}

fn merged_json_target(source: &Value, destination: &Path) -> Result<Value> {
    if !destination.is_file() {
        return Ok(source.clone());
    }
    let contents = fs::read_to_string(destination)
        .with_context(|| format!("failed to read {}", destination.display()))?;
    let mut destination_value: Value = serde_json::from_str(&contents)
        .with_context(|| format!("invalid JSON in {}", destination.display()))?;
    merge_json(&mut destination_value, source.clone());
    Ok(destination_value)
}

fn merge_json(destination: &mut Value, source: Value) {
    match (destination, source) {
        (Value::Object(destination), Value::Object(source)) => {
            for (key, value) in source {
                merge_json(destination.entry(key).or_insert(Value::Null), value);
            }
        }
        (destination, source) => {
            *destination = source;
        }
    }
}

fn build_codex_mcp_config(
    source: &Value,
    destination: &Path,
    request: &InstallRequest,
) -> Result<String> {
    let mut document = if destination.is_file() {
        let contents = fs::read_to_string(destination)
            .with_context(|| format!("failed to read {}", destination.display()))?;
        contents
            .parse::<DocumentMut>()
            .with_context(|| format!("invalid TOML in {}", destination.display()))?
    } else {
        DocumentMut::new()
    };

    let servers = source
        .get(MCP_SERVERS_KEY)
        .and_then(Value::as_object)
        .context("MCP config missing mcpServers object")?;
    let mcp_servers = ensure_toml_table(&mut document, MCP_SERVERS_TOML_KEY)?;
    for (name, server) in servers {
        if mcp_servers.contains_key(name) && request.conflict_mode == ConflictMode::Skip {
            continue;
        }
        mcp_servers[name] = codex_mcp_server_table(server)?;
    }

    let mut serialized = document.to_string();
    if !serialized.ends_with('\n') {
        serialized.push('\n');
    }
    Ok(serialized)
}

fn ensure_toml_table<'document>(
    document: &'document mut DocumentMut,
    key: &str,
) -> Result<&'document mut TomlTable> {
    if !document.as_table().contains_key(key) {
        document[key] = toml_edit::table();
    }
    document[key]
        .as_table_mut()
        .with_context(|| format!("{key} exists but is not a TOML table"))
}

fn codex_mcp_server_table(server: &Value) -> Result<TomlItem> {
    let object = server
        .as_object()
        .context("MCP server entry must be a JSON object")?;
    let mut table = TomlTable::new();
    if let Some(command) = object.get("command").and_then(Value::as_str) {
        table["command"] = toml_value(command);
    }
    if let Some(args) = object.get("args").and_then(Value::as_array) {
        let mut array = toml_edit::Array::default();
        for arg in args {
            let arg = arg
                .as_str()
                .context("MCP server args entries must be strings")?;
            array.push(arg);
        }
        table["args"] = TomlItem::Value(toml_edit::Value::Array(array));
    }
    if let Some(server_type) = object.get("type").and_then(Value::as_str) {
        table["type"] = toml_value(server_type);
    }
    if let Some(url) = object.get("url").and_then(Value::as_str) {
        table["url"] = toml_value(url);
    }
    Ok(TomlItem::Table(table))
}

fn format_json(value: &Value) -> Result<String> {
    let mut serialized = serde_json::to_string_pretty(value).context("failed to serialize JSON")?;
    serialized.push('\n');
    Ok(serialized)
}

fn copy_tree(
    source: &Path,
    destination: &Path,
    exclude_prefix: Option<&Path>,
    directory_conflict_mode: DirectoryConflictMode,
    request: &InstallRequest,
    plan: &mut InstallPlan,
) -> Result<()> {
    if !source.is_dir() {
        anyhow::bail!("missing source directory: {}", source.display());
    }

    let source_files = collect_source_files(source, exclude_prefix)?;
    let destination_will_be_replaced = if directory_conflict_mode
        == DirectoryConflictMode::WholeDirectory
        && is_nonempty_directory(destination)?
    {
        if source_tree_matches_destination(source, destination, &source_files)? {
            return Ok(());
        }
        if effective_conflict_mode(request) == ConflictMode::Selective {
            false
        } else if !plan_directory_conflict(request, plan, destination) {
            return Ok(());
        } else {
            true
        }
    } else {
        false
    };

    for source_file in source_files {
        let relative_path = source_file.strip_prefix(source).with_context(|| {
            format!(
                "failed to derive relative path for {}",
                source_file.display()
            )
        })?;
        let destination_file = destination.join(relative_path);
        plan_copy_file_with_state(
            request,
            plan,
            source_file,
            destination_file,
            destination_will_be_replaced,
        )?;
    }

    Ok(())
}

fn source_tree_matches_destination(
    source: &Path,
    destination: &Path,
    source_files: &[PathBuf],
) -> Result<bool> {
    if !destination.is_dir() {
        return Ok(false);
    }

    let destination_files = collect_source_files(destination, None)?;
    if source_files.len() != destination_files.len() {
        return Ok(false);
    }

    for source_file in source_files {
        let relative_path = source_file.strip_prefix(source).with_context(|| {
            format!(
                "failed to derive relative path for {}",
                source_file.display()
            )
        })?;
        let destination_file = destination.join(relative_path);
        if !destination_file.is_file() || !files_have_same_contents(source_file, &destination_file)?
        {
            return Ok(false);
        }
    }

    Ok(true)
}

fn collect_source_files(source: &Path, exclude_prefix: Option<&Path>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_source_files_inner(source, exclude_prefix, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_source_files_inner(
    source: &Path,
    exclude_prefix: Option<&Path>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry under {}", source.display()))?;
        let path = entry.path();
        if exclude_prefix.is_some_and(|prefix| path.starts_with(prefix)) {
            continue;
        }
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", path.display()))?;
        if file_type.is_dir() {
            collect_source_files_inner(&path, exclude_prefix, files)?;
        } else if file_type.is_file() && entry.file_name() != METADATA_FILE_NAME {
            files.push(path);
        }
    }
    Ok(())
}

fn plan_copy_file_with_state(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    source: PathBuf,
    destination: PathBuf,
    destination_will_be_replaced: bool,
) -> Result<()> {
    if !destination_will_be_replaced
        && destination.is_file()
        && files_have_same_contents(&source, &destination)?
    {
        return Ok(());
    }
    if !destination_will_be_replaced
        && destination.exists()
        && !plan_file_conflict(request, plan, &destination)
    {
        return Ok(());
    }
    plan.operations.push(InstallOperation::CopyFile {
        source,
        destination,
    });
    Ok(())
}

fn plan_write_file(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    destination: PathBuf,
    contents: String,
) -> Result<()> {
    if destination.is_file() && file_has_contents(&destination, contents.as_bytes())? {
        return Ok(());
    }
    if destination.exists() && !plan_file_conflict(request, plan, &destination) {
        return Ok(());
    }
    plan.operations.push(InstallOperation::WriteFile {
        destination,
        contents,
    });
    Ok(())
}

fn append_rules_summary(
    source: &Path,
    destination: &Path,
    request: &InstallRequest,
    plan: &mut InstallPlan,
) -> Result<()> {
    if !source.is_file() {
        return Ok(());
    }
    if destination.is_file() {
        let existing = fs::read_to_string(destination)
            .with_context(|| format!("failed to read {}", destination.display()))?;
        if existing.contains(RULES_SUMMARY_MARKER) {
            return Ok(());
        }
    }
    let contents = fs::read_to_string(source)
        .with_context(|| format!("failed to read {}", source.display()))?;
    let contents = if destination.is_file() {
        format!("\n\n{contents}")
    } else {
        contents
    };
    let operation = if destination.is_file() {
        InstallOperation::AppendFile {
            destination: destination.to_path_buf(),
            contents,
        }
    } else {
        InstallOperation::WriteFile {
            destination: destination.to_path_buf(),
            contents,
        }
    };
    if destination.exists() {
        match request.conflict_mode {
            ConflictMode::Skip => {
                plan.operations.push(InstallOperation::Skip {
                    target: destination.to_path_buf(),
                    reason: "conflict mode is skip".to_string(),
                });
                return Ok(());
            }
            ConflictMode::Selective => {
                plan.operations.push(InstallOperation::PromptConflict {
                    target: destination.to_path_buf(),
                    choices: "append or skip".to_string(),
                });
            }
            ConflictMode::Overwrite | ConflictMode::Backup => {}
        }
    }
    plan.operations.push(operation);
    Ok(())
}

fn plan_directory_conflict(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    destination: &Path,
) -> bool {
    match effective_conflict_mode(request) {
        ConflictMode::Skip => {
            plan.operations.push(InstallOperation::Skip {
                target: destination.to_path_buf(),
                reason: "conflict mode is skip".to_string(),
            });
            false
        }
        ConflictMode::Backup => {
            plan.operations.push(InstallOperation::BackupPath {
                path: destination.to_path_buf(),
                backup: backup_path(destination, &plan_backup_timestamp(request)),
            });
            true
        }
        ConflictMode::Overwrite => {
            plan.operations.push(InstallOperation::RemovePath {
                path: destination.to_path_buf(),
            });
            true
        }
        ConflictMode::Selective => true,
    }
}

fn plan_file_conflict(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    destination: &Path,
) -> bool {
    match effective_conflict_mode(request) {
        ConflictMode::Skip => {
            plan.operations.push(InstallOperation::Skip {
                target: destination.to_path_buf(),
                reason: "conflict mode is skip".to_string(),
            });
            false
        }
        ConflictMode::Backup => {
            plan.operations.push(InstallOperation::BackupPath {
                path: destination.to_path_buf(),
                backup: backup_path(destination, &plan_backup_timestamp(request)),
            });
            true
        }
        ConflictMode::Selective => {
            plan.operations.push(InstallOperation::PromptConflict {
                target: destination.to_path_buf(),
                choices: "backup, overwrite, or skip".to_string(),
            });
            true
        }
        ConflictMode::Overwrite => {
            if destination.is_dir() {
                plan.operations.push(InstallOperation::RemovePath {
                    path: destination.to_path_buf(),
                });
            }
            true
        }
    }
}

fn execute_operation(operation: &InstallOperation, request: &InstallRequest) -> Result<()> {
    match operation {
        InstallOperation::CopyFile {
            source,
            destination,
        } => {
            let backup = backup_path(destination, &plan_backup_timestamp(request));
            if !prepare_existing_target_for_write(destination, request.conflict_mode, &backup)? {
                return Ok(());
            }
            ensure_parent(destination)?;
            fs::copy(source, destination).with_context(|| {
                format!(
                    "failed to copy {} -> {}",
                    source.display(),
                    destination.display()
                )
            })?;
        }
        InstallOperation::BackupPath { path, backup } => {
            ensure_parent(backup)?;
            fs::rename(path, backup).with_context(|| {
                format!(
                    "failed to backup {} -> {}",
                    path.display(),
                    backup.display()
                )
            })?;
        }
        InstallOperation::RemovePath { path } => {
            if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            }
            .with_context(|| format!("failed to remove {}", path.display()))?;
        }
        InstallOperation::WriteFile {
            destination,
            contents,
        } => {
            let backup = backup_path(destination, &plan_backup_timestamp(request));
            if !prepare_existing_target_for_write(destination, request.conflict_mode, &backup)? {
                return Ok(());
            }
            ensure_parent(destination)?;
            fs::write(destination, contents)
                .with_context(|| format!("failed to write {}", destination.display()))?;
        }
        InstallOperation::AppendFile {
            destination,
            contents,
        } => {
            if !prepare_existing_target_for_append(destination, request.conflict_mode)? {
                return Ok(());
            }
            ensure_parent(destination)?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(destination)
                .with_context(|| format!("failed to open {}", destination.display()))?;
            file.write_all(contents.as_bytes())
                .with_context(|| format!("failed to append {}", destination.display()))?;
        }
        InstallOperation::PromptConflict { .. } => {}
        InstallOperation::RunCommand { program, args } => {
            let status = Command::new(program)
                .args(args)
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| format!("failed to run {}", shell_words(program, args)))?;
            if !status.success() {
                anyhow::bail!(
                    "{} failed with status {:?}",
                    shell_words(program, args),
                    status.code()
                );
            }
        }
        InstallOperation::UpdateManifest {
            path,
            upsert,
            remove_ids,
        } => {
            apply_manifest_update(path, upsert, remove_ids)?;
        }
        InstallOperation::Skip { .. } => {}
    }
    Ok(())
}

fn shell_words(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .map(|part| shell_quote(&part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    if value.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || matches!(character, '/' | '.' | '-' | '_' | '@' | ':' | '=')
    }) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Ok(())
}

fn files_have_same_contents(left: &Path, right: &Path) -> Result<bool> {
    let left_metadata =
        fs::metadata(left).with_context(|| format!("failed to inspect {}", left.display()))?;
    let right_metadata =
        fs::metadata(right).with_context(|| format!("failed to inspect {}", right.display()))?;
    if left_metadata.len() != right_metadata.len() {
        return Ok(false);
    }
    let left_contents =
        fs::read(left).with_context(|| format!("failed to read {}", left.display()))?;
    let right_contents =
        fs::read(right).with_context(|| format!("failed to read {}", right.display()))?;
    Ok(left_contents == right_contents)
}

fn file_has_contents(path: &Path, expected_contents: &[u8]) -> Result<bool> {
    let metadata =
        fs::metadata(path).with_context(|| format!("failed to inspect {}", path.display()))?;
    if metadata.len() != expected_contents.len() as u64 {
        return Ok(false);
    }
    let actual_contents =
        fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(actual_contents == expected_contents)
}

fn is_nonempty_directory(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }
    Ok(fs::read_dir(path)
        .with_context(|| format!("failed to read {}", path.display()))?
        .next()
        .transpose()
        .with_context(|| format!("failed to read entry under {}", path.display()))?
        .is_some())
}

fn rules_root_for(
    editor: Editor,
    scope: Scope,
    target_root: &Path,
    context: &InstallContext,
) -> PathBuf {
    if editor == Editor::Cursor {
        return target_root.join(CURSOR_RULES_DIR);
    }
    match scope {
        Scope::Global => context.home.join(GLOBAL_AGENT_RULES_DIR),
        Scope::Project => context
            .project_root
            .join(DOT_AGENT_DIR)
            .join(CURSOR_RULES_DIR),
    }
}

fn rules_summary_target_for(
    editor: Editor,
    scope: Scope,
    target_root: &Path,
    context: &InstallContext,
) -> PathBuf {
    if editor == Editor::Claude {
        if scope == Scope::Project {
            let claude_file = context.project_root.join(CLAUDE_FILE);
            if file_has_more_than_lines(&claude_file, DEFAULT_PROJECT_CLAUDE_RULE_LIMIT) {
                return claude_file;
            }
            return context.project_root.join(AGENTS_FILE);
        }
        return target_root.join(CLAUDE_FILE);
    }

    match scope {
        Scope::Global => target_root.join(AGENTS_FILE),
        Scope::Project => context.project_root.join(AGENTS_FILE),
    }
}

fn file_has_more_than_lines(path: &Path, limit: usize) -> bool {
    fs::read_to_string(path)
        .map(|contents| contents.lines().take(limit + 1).count() > limit)
        .unwrap_or(false)
}

fn skills_root_for(editor: Editor, scope: Scope, target_root: &Path) -> Result<PathBuf> {
    if editor == Editor::Ampcode && scope == Scope::Global {
        return Ok(home_dir()?.join(GLOBAL_AMP_SKILLS_DIR));
    }
    Ok(target_root.join(SKILLS_DIR))
}

fn shares_skills_root(editor: Editor) -> bool {
    matches!(editor, Editor::Claude | Editor::Codex | Editor::Ampcode)
}

fn target_root_for(editor: Editor, scope: Scope, context: &InstallContext) -> Result<PathBuf> {
    let root = match (editor, scope) {
        (Editor::Cursor, Scope::Global) => context.home.join(GLOBAL_CURSOR_DIR),
        (Editor::Cursor, Scope::Project) => context.project_root.join(DOT_CURSOR_DIR),
        (Editor::Claude, Scope::Global) => context.home.join(GLOBAL_CLAUDE_DIR),
        (Editor::Claude, Scope::Project) => context.project_root.join(DOT_CLAUDE_DIR),
        (Editor::Opencode, Scope::Global) => context.home.join(GLOBAL_OPENCODE_DIR),
        (Editor::Opencode, Scope::Project) => context.project_root.join(DOT_OPENCODE_DIR),
        (Editor::Codex, Scope::Global) => context.home.join(GLOBAL_CODEX_DIR),
        (Editor::Codex, Scope::Project) => context.project_root.join(DOT_CODEX_DIR),
        (Editor::Ampcode, Scope::Global) => context.home.join(GLOBAL_AMP_DIR),
        (Editor::Ampcode, Scope::Project) => context.project_root.join(DOT_AGENT_DIR),
    };
    Ok(root)
}

fn mcp_path_for(
    editor: Editor,
    scope: Scope,
    target_root: &Path,
    context: &InstallContext,
) -> PathBuf {
    match (editor, scope) {
        (Editor::Claude, Scope::Global) => context.home.join(GLOBAL_CLAUDE_MCP_FILE),
        (Editor::Claude, Scope::Project) => context.project_root.join(DOT_MCP_FILE),
        (Editor::Cursor, _) => target_root.join("mcp.json"),
        (Editor::Opencode, Scope::Global) => target_root.join(DOT_OPENCODE_FILE),
        (Editor::Opencode, Scope::Project) => context.project_root.join(DOT_OPENCODE_FILE),
        (Editor::Codex, _) => target_root.join(CODEX_CONFIG_FILE),
        (Editor::Ampcode, _) => target_root.join("settings.json"),
    }
}

fn effective_conflict_mode(request: &InstallRequest) -> ConflictMode {
    request.conflict_mode
}

fn plan_backup_timestamp(request: &InstallRequest) -> String {
    if request.dry_run {
        "timestamp".to_string()
    } else {
        backup_timestamp()
    }
}

fn backup_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn backup_path(path: &Path, timestamp: &str) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(format!(".{BACKUP_EXTENSION_PREFIX}.{timestamp}"));
    PathBuf::from(backup)
}

fn home_dir() -> Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .context("HOME is not set")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("test clock should be valid")
                .as_nanos();
            let path = env::temp_dir().join(format!("stacc-{name}-{unique}"));
            fs::create_dir_all(&path).expect("test dir should be created");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn write_test_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("test parent should be created");
        }
        fs::write(path, contents).expect("test file should be written");
    }

    fn dry_run_request_with_root(root: PathBuf, categories: Vec<Category>) -> InstallRequest {
        InstallRequest {
            root,
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            categories,
            stacks: Vec::new(),
            mcp_servers: Vec::new(),
            hook_packages: Vec::new(),
            codex_plugins: Vec::new(),
            conflict_mode: ConflictMode::Backup,
            yes: true,
            dry_run: true,
        }
    }

    fn dry_run_request(categories: Vec<Category>) -> InstallRequest {
        dry_run_request_with_root(PathBuf::from("."), categories)
    }

    fn test_plan(target_root: PathBuf) -> InstallPlan {
        InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Project,
            target_root,
            operations: Vec::new(),
        }
    }

    #[test]
    fn filters_codex_project_mcp_category() {
        let request = dry_run_request(vec![Category::Skills, Category::Mcps]);
        let categories = filtered_categories(&request, Editor::Codex);
        assert_eq!(categories, vec![Category::Skills]);
    }

    #[test]
    fn builds_ampcode_global_mcp_settings_plan() {
        let root = TestDir::new("amp-mcp");
        write_test_file(
            &root.path.join(CONFIGS_DIR).join("mcps").join("mcp.json"),
            r#"{"mcpServers":{"github":{"type":"http","url":"https://example.test"}}}"#,
        );
        let mut request = dry_run_request_with_root(root.path.clone(), vec![Category::Mcps]);
        request.editors = vec![Editor::Ampcode];
        request.scope = Scope::Global;

        let plans = build_install_plan(&request).expect("AMP MCP plan should build");

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].editor, Editor::Ampcode);
        assert_eq!(plans[0].scope, Scope::Global);
        assert!(plans[0].operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::WriteFile {
                destination,
                contents
            } if destination.ends_with(".config/amp/settings.json")
                && contents.contains("\"amp\"")
                && contents.contains("\"mcpServers\"")
                && contents.contains("\"github\"")
        )));
    }

    #[test]
    fn deep_merges_json_objects() {
        let mut destination = serde_json::json!({
            "mcpServers": {
                "github": {"url": "old"},
                "grep": {"url": "keep"}
            },
            "other": true
        });
        let source = serde_json::json!({
            "mcpServers": {
                "github": {"url": "new"}
            }
        });

        merge_json(&mut destination, source);

        assert_eq!(destination["mcpServers"]["github"]["url"], "new");
        assert_eq!(destination["mcpServers"]["grep"]["url"], "keep");
        assert_eq!(destination["other"], true);
    }

    #[test]
    fn builds_codex_mcp_toml() {
        let request = dry_run_request(vec![Category::Mcps]);
        let source = serde_json::json!({
            "mcpServers": {
                "github": {
                    "type": "http",
                    "url": "https://api.githubcopilot.com/mcp/"
                }
            }
        });

        let rendered = build_codex_mcp_config(&source, Path::new("missing.toml"), &request)
            .expect("codex config should render");

        assert!(rendered.contains("[mcp_servers.github]"));
        assert!(rendered.contains("type = \"http\""));
        assert!(rendered.contains("url = \"https://api.githubcopilot.com/mcp/\""));
    }

    #[test]
    fn build_plan_uses_native_operations() {
        let mut request = dry_run_request(vec![Category::Rules]);
        request.editors = vec![Editor::Cursor];

        let plans = build_install_plan(&request).expect("plan should build");

        assert_eq!(plans.len(), 1);
        assert!(plans[0]
            .operations
            .iter()
            .any(|operation| matches!(operation, InstallOperation::CopyFile { .. })));
    }

    #[test]
    fn selected_stack_names_reject_unknown_values() {
        let root = TestDir::new("unknown-stack");
        fs::create_dir_all(root.path.join(STACKS_DIR).join("rust"))
            .expect("stack dir should be created");
        let mut request = dry_run_request_with_root(root.path.clone(), vec![Category::Stack]);
        request.stacks = vec!["../rust".to_string(), "missing".to_string()];

        let error = selected_stack_names(&request).expect_err("unknown stacks should fail");

        let message = error.to_string();
        assert!(message.contains("unknown stack(s): ../rust,missing"));
        assert!(message.contains("Available: rust"));
    }

    #[test]
    fn selected_stack_names_requires_all_to_stand_alone() {
        let root = TestDir::new("all-stack");
        fs::create_dir_all(root.path.join(STACKS_DIR).join("rust"))
            .expect("stack dir should be created");
        let mut request = dry_run_request_with_root(root.path.clone(), vec![Category::Stack]);
        request.stacks = vec!["all".to_string(), "rust".to_string()];

        let error = selected_stack_names(&request).expect_err("mixed all should fail");

        assert!(error
            .to_string()
            .contains("--stack all cannot be combined with specific stack names"));
    }

    #[test]
    fn selected_mcp_config_rejects_partial_unknown_servers() {
        let root = TestDir::new("unknown-mcp");
        write_test_file(
            &root.path.join(CONFIGS_DIR).join("mcps").join("mcp.json"),
            r#"{"mcpServers":{"github":{"type":"http","url":"https://example.test"}}}"#,
        );
        let mut request = dry_run_request_with_root(root.path.clone(), vec![Category::Mcps]);
        request.mcp_servers = vec!["github".to_string(), "missing".to_string()];

        let error = selected_mcp_config(&request).expect_err("unknown MCP server should fail");

        let message = error.to_string();
        assert!(message.contains("unknown MCP server(s): missing"));
        assert!(message.contains("Available: github"));
    }

    #[test]
    fn builds_codex_plugin_command_plan() {
        let root = TestDir::new("codex-plugin");
        write_codex_plugin_catalog(root.path.as_path());
        let mut request =
            dry_run_request_with_root(root.path.clone(), vec![Category::CodexPlugins]);
        request.codex_plugins = vec!["lazycodex".to_string()];

        let plans = build_install_plan(&request).expect("Codex plugin plan should build");

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].scope, Scope::Global);
        assert!(plans[0].operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args
                        == &[
                            "plugin".to_string(),
                            "marketplace".to_string(),
                            "add".to_string(),
                            "code-yeongyu/lazycodex".to_string()
                        ]
        )));
        assert!(plans[0].operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args == &["plugin".to_string(), "add".to_string(), "omo@sisyphuslabs".to_string()]
        )));
    }

    #[test]
    fn selected_codex_plugins_rejects_unknown_values() {
        let root = TestDir::new("unknown-codex-plugin");
        write_codex_plugin_catalog(root.path.as_path());
        let mut request =
            dry_run_request_with_root(root.path.clone(), vec![Category::CodexPlugins]);
        request.codex_plugins = vec!["missing".to_string()];

        let error = selected_codex_plugins(&request).expect_err("unknown plugin should fail");

        let message = error.to_string();
        assert!(message.contains("unknown Codex plugin(s): missing"));
        assert!(message.contains(CODEX_PLUGIN_INSTALL_EXAMPLE));
        assert!(message.contains("Available: lazycodex"));
    }

    #[test]
    fn selected_codex_plugins_requires_explicit_selection() {
        let root = TestDir::new("empty-codex-plugin");
        write_codex_plugin_catalog(root.path.as_path());
        let request = dry_run_request_with_root(root.path.clone(), vec![Category::CodexPlugins]);

        let error = selected_codex_plugins(&request).expect_err("empty selection should fail");

        let message = error.to_string();
        assert!(message.contains("codex-plugins requires --codex-plugin"));
        assert!(message.contains(CODEX_PLUGIN_INSTALL_EXAMPLE));
        assert!(message.contains("Available: lazycodex"));
    }

    #[test]
    fn codex_plugin_flag_implies_category() {
        let root = TestDir::new("implied-codex-plugin-category");
        write_codex_plugin_catalog(root.path.as_path());
        let mut request = dry_run_request(vec![Category::Skills]);
        request.root = root.path.clone();
        request.codex_plugins = vec!["lazycodex".to_string()];

        let plans = build_install_plan(&request).expect("plugin flag should imply plugin category");

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].scope, Scope::Global);
        assert!(plans[0].operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args
                        == &[
                            "plugin".to_string(),
                            "marketplace".to_string(),
                            "add".to_string(),
                            "code-yeongyu/lazycodex".to_string()
                        ]
        )));
    }

    #[test]
    fn codex_plugin_category_requires_codex_editor_and_assumes_global() {
        let root = TestDir::new("codex-plugin-global-scope");
        write_codex_plugin_catalog(root.path.as_path());
        let mut request = dry_run_request(vec![Category::CodexPlugins]);
        request.root = root.path.clone();
        request.editors = vec![Editor::Cursor];
        request.scope = Scope::Project;
        request.codex_plugins = vec!["lazycodex".to_string()];

        let error = build_install_plan(&request).expect_err("plugin category needs Codex");

        let message = error.to_string();
        assert!(message.contains("codex-plugins requires --editor codex"));
        assert!(message.contains(CODEX_PLUGIN_INSTALL_EXAMPLE));

        request.editors = vec![Editor::Codex];
        let plans = build_install_plan(&request).expect("plugin category should assume global");

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].scope, Scope::Global);
    }

    #[test]
    fn manifest_update_upserts_and_removes_entries() {
        let root = TestDir::new("manifest-update");
        let path = root
            .path
            .join(".codex")
            .join(".stacc")
            .join("manifest.json");
        let entry = managed_skill_entry_for_test(
            "demo",
            &root.path.join(".codex").join("skills").join("demo"),
        );

        apply_manifest_update(&path, std::slice::from_ref(&entry), &[])
            .expect("manifest upsert should succeed");
        let manifest = read_manifest(&path).expect("manifest should read");

        assert_eq!(manifest.entries, vec![entry.clone()]);

        apply_manifest_update(&path, &[], std::slice::from_ref(&entry.id))
            .expect("manifest remove should succeed");
        let manifest = read_manifest(&path).expect("manifest should read after remove");

        assert!(manifest.entries.is_empty());
    }

    #[test]
    fn skill_manifest_entry_prefers_relative_destination() {
        let root = TestDir::new("manifest-relative-destination");
        let request = dry_run_request_with_root(root.path.clone(), vec![Category::Skills]);
        let plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Project,
            target_root: root.path.join(".codex"),
            operations: Vec::new(),
        };
        let source = root.path.join(CONFIGS_DIR).join("skills").join("demo");
        let destination = plan.target_root.join("skills").join("demo");

        let entry = skill_manifest_entry(
            &request,
            &plan,
            Category::Skills,
            "demo",
            &source,
            &destination,
        )
        .expect("manifest entry should build");

        assert_eq!(entry.source.as_deref(), Some("configs/skills/demo"));
        assert_eq!(
            entry.destination,
            Some(PathBuf::from("skills").join("demo"))
        );
    }

    #[test]
    fn managed_skill_update_uses_manifest_source_and_destination() {
        let root = TestDir::new("managed-skill-update");
        let source = root.path.join(CONFIGS_DIR).join("skills").join("demo");
        let target_root = root.path.join(".codex");
        let destination = target_root.join("skills").join("demo");
        write_test_file(&source.join(MANAGED_SKILL_FILE_NAME), "demo skill");
        let entry = managed_skill_entry_for_test("demo", &destination);
        apply_manifest_update(
            &manifest_path_for(&target_root),
            std::slice::from_ref(&entry),
            &[],
        )
        .expect("manifest upsert should succeed");
        let request = manage_request_for_test(root.path.clone(), vec!["demo".to_string()]);
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Project,
            target_root: target_root.clone(),
            operations: Vec::new(),
        };

        plan_managed_skills(&request, &mut plan, &request.skills, ManageAction::Update)
            .expect("managed skill update should plan");

        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::CopyFile {
                source: copy_source,
                destination: copy_destination
            } if copy_source == &source.join(MANAGED_SKILL_FILE_NAME)
                && copy_destination == &destination.join(MANAGED_SKILL_FILE_NAME)
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::UpdateManifest { upsert, remove_ids, .. }
                if upsert.len() == 1
                    && upsert[0].id == entry.id
                    && remove_ids.is_empty()
        )));
    }

    #[test]
    fn sync_manifest_backfills_installed_skill() {
        let root = TestDir::new("sync-skill");
        let source = root.path.join(CONFIGS_DIR).join("skills").join("demo");
        let target_root = root.path.join(".codex");
        let destination = target_root.join("skills").join("demo");
        write_test_file(&source.join(MANAGED_SKILL_FILE_NAME), "source demo");
        write_test_file(&destination.join(MANAGED_SKILL_FILE_NAME), "installed demo");
        let request = SyncManifestRequest {
            root: root.path.clone(),
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            skills: vec!["demo".to_string()],
            codex_plugins: Vec::new(),
            yes: true,
            dry_run: true,
        };
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Project,
            target_root,
            operations: Vec::new(),
        };

        plan_manifest_skill_sync(&request, &mut plan, &request.skills)
            .expect("manifest skill sync should plan");

        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::UpdateManifest { upsert, remove_ids, .. }
                if upsert.len() == 1
                    && upsert[0].id == "codex:project:skill:demo"
                    && upsert[0].source.as_deref() == Some("configs/skills/demo")
                    && upsert[0].destination == Some(PathBuf::from("skills").join("demo"))
                    && remove_ids.is_empty()
        )));
    }

    #[test]
    fn sync_manifest_backfills_selected_codex_plugin() {
        let root = TestDir::new("sync-plugin");
        write_codex_plugin_catalog(root.path.as_path());
        let request = SyncManifestRequest {
            root: root.path.clone(),
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            skills: Vec::new(),
            codex_plugins: vec!["lazycodex".to_string()],
            yes: true,
            dry_run: true,
        };
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Global,
            target_root: root.path.join(".codex"),
            operations: Vec::new(),
        };

        plan_manifest_codex_plugin_sync(&request, &mut plan, &request.codex_plugins)
            .expect("manifest plugin sync should plan");

        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::UpdateManifest { upsert, remove_ids, .. }
                if upsert.len() == 1
                    && upsert[0].id == "codex:global:codex-plugin:lazycodex"
                    && upsert[0].selector.as_deref() == Some("omo@sisyphuslabs")
                    && upsert[0].marketplace.as_deref() == Some("sisyphuslabs")
                    && remove_ids.is_empty()
        )));
    }

    #[test]
    fn managed_skill_uninstall_uses_manifest_destination() {
        let root = TestDir::new("managed-skill-uninstall");
        let source = root.path.join(CONFIGS_DIR).join("skills").join("demo");
        let target_root = root.path.join(".codex");
        let destination = target_root.join("skills").join("demo");
        write_test_file(&source.join(MANAGED_SKILL_FILE_NAME), "demo skill");
        write_test_file(&destination.join(MANAGED_SKILL_FILE_NAME), "installed demo");
        let entry = managed_skill_entry_for_test("demo", &destination);
        apply_manifest_update(
            &manifest_path_for(&target_root),
            std::slice::from_ref(&entry),
            &[],
        )
        .expect("manifest upsert should succeed");
        let request = manage_request_for_test(root.path.clone(), vec!["demo".to_string()]);
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Project,
            target_root,
            operations: Vec::new(),
        };

        plan_managed_skills(
            &request,
            &mut plan,
            &request.skills,
            ManageAction::Uninstall,
        )
        .expect("managed skill uninstall should plan");

        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RemovePath { path } if path == &destination
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::UpdateManifest { upsert, remove_ids, .. }
                if upsert.is_empty() && remove_ids == &vec![entry.id.clone()]
        )));
    }

    #[test]
    fn managed_codex_plugin_uninstall_removes_plugin_and_unused_marketplace() {
        let root = TestDir::new("managed-plugin-uninstall");
        let target_root = root.path.join(".codex");
        let selected = lazycodex_selected_plugin();
        let entry = codex_plugin_manifest_entry_from_parts(true, &selected);
        apply_manifest_update(
            &manifest_path_for(&target_root),
            std::slice::from_ref(&entry),
            &[],
        )
        .expect("manifest upsert should succeed");
        let request = ManageRequest {
            root: root.path.clone(),
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            skills: Vec::new(),
            codex_plugins: vec!["lazycodex".to_string()],
            conflict_mode: ConflictMode::Backup,
            yes: true,
            dry_run: true,
        };
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Global,
            target_root,
            operations: Vec::new(),
        };

        plan_managed_codex_plugins(
            &request,
            &mut plan,
            &request.codex_plugins,
            ManageAction::Uninstall,
        )
        .expect("managed Codex plugin uninstall should plan");

        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args == &["plugin".to_string(), "remove".to_string(), "omo@sisyphuslabs".to_string()]
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args
                        == &[
                            "plugin".to_string(),
                            "marketplace".to_string(),
                            "remove".to_string(),
                            "sisyphuslabs".to_string()
                        ]
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::UpdateManifest { upsert, remove_ids, .. }
                if upsert.is_empty() && remove_ids == &vec![entry.id.clone()]
        )));
    }

    #[test]
    fn managed_codex_plugin_update_refreshes_marketplace_and_plugin() {
        let root = TestDir::new("managed-plugin-update");
        write_codex_plugin_catalog(root.path.as_path());
        let target_root = root.path.join(".codex");
        let selected = lazycodex_selected_plugin();
        let entry = codex_plugin_manifest_entry_from_parts(true, &selected);
        apply_manifest_update(
            &manifest_path_for(&target_root),
            std::slice::from_ref(&entry),
            &[],
        )
        .expect("manifest upsert should succeed");
        let request = ManageRequest {
            root: root.path.clone(),
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            skills: Vec::new(),
            codex_plugins: vec!["lazycodex".to_string()],
            conflict_mode: ConflictMode::Backup,
            yes: true,
            dry_run: true,
        };
        let mut plan = InstallPlan {
            editor: Editor::Codex,
            scope: Scope::Global,
            target_root,
            operations: Vec::new(),
        };

        plan_managed_codex_plugins(
            &request,
            &mut plan,
            &request.codex_plugins,
            ManageAction::Update,
        )
        .expect("managed Codex plugin update should plan");

        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args
                        == &[
                            "plugin".to_string(),
                            "marketplace".to_string(),
                            "add".to_string(),
                            "code-yeongyu/lazycodex".to_string()
                        ]
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args
                        == &[
                            "plugin".to_string(),
                            "marketplace".to_string(),
                            "upgrade".to_string(),
                            "sisyphuslabs".to_string()
                        ]
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::RunCommand { program, args }
                if program == CODEX_COMMAND
                    && args == &["plugin".to_string(), "add".to_string(), "omo@sisyphuslabs".to_string()]
        )));
        assert!(plan.operations.iter().any(|operation| matches!(
            operation,
            InstallOperation::UpdateManifest { upsert, remove_ids, .. }
                if upsert.len() == 1
                    && upsert[0].id == entry.id
                    && remove_ids.is_empty()
        )));
    }

    fn manage_request_for_test(root: PathBuf, skills: Vec<String>) -> ManageRequest {
        ManageRequest {
            root,
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            skills,
            codex_plugins: Vec::new(),
            conflict_mode: ConflictMode::Backup,
            yes: true,
            dry_run: true,
        }
    }

    fn managed_skill_entry_for_test(name: &str, _destination: &Path) -> ManagedEntry {
        ManagedEntry {
            id: managed_entry_id(ManagedKind::Skill, Editor::Codex, Scope::Project, name),
            kind: ManagedKind::Skill,
            name: name.to_string(),
            editor: Editor::Codex,
            scope: Scope::Project,
            category: Some(Category::Skills),
            source: Some("configs/skills/demo".to_string()),
            destination: Some(PathBuf::from("skills").join(name)),
            selector: None,
            marketplace: None,
            marketplace_source: None,
            source_url: None,
            license: None,
            updated_unix_seconds: 0,
        }
    }

    fn lazycodex_selected_plugin() -> SelectedCodexPlugin {
        SelectedCodexPlugin {
            name: "lazycodex".to_string(),
            spec: CodexPluginSpec {
                description: "LazyCodex marketplace plugin".to_string(),
                marketplace_source: "code-yeongyu/lazycodex".to_string(),
                marketplace: "sisyphuslabs".to_string(),
                plugin: "omo@sisyphuslabs".to_string(),
                source_url: "https://github.com/code-yeongyu/lazycodex".to_string(),
                license: "MIT".to_string(),
            },
        }
    }

    fn write_codex_plugin_catalog(root: &Path) {
        write_test_file(
            &root
                .join(CONFIGS_DIR)
                .join("codex-plugins")
                .join(CODEX_PLUGINS_CONFIG_FILE),
            r#"{
              "plugins": {
                "lazycodex": {
                  "description": "LazyCodex marketplace plugin",
                  "marketplace_source": "code-yeongyu/lazycodex",
                  "marketplace": "sisyphuslabs",
                  "plugin": "omo@sisyphuslabs",
                  "source_url": "https://github.com/code-yeongyu/lazycodex",
                  "license": "MIT"
                }
              }
            }"#,
        );
    }

    #[test]
    fn identical_copy_file_is_noop() {
        let root = TestDir::new("copy-noop");
        let source = root.path.join("source.txt");
        let destination = root.path.join("destination.txt");
        write_test_file(&source, "same");
        write_test_file(&destination, "same");
        let request = dry_run_request(vec![Category::Rules]);
        let mut plan = test_plan(root.path.clone());

        plan_copy_file_with_state(&request, &mut plan, source, destination, false)
            .expect("copy planning should succeed");

        assert!(plan.operations.is_empty());
    }

    #[test]
    fn identical_write_file_is_noop() {
        let root = TestDir::new("write-noop");
        let destination = root.path.join("mcp.json");
        write_test_file(&destination, "{}\n");
        let request = dry_run_request(vec![Category::Mcps]);
        let mut plan = test_plan(root.path.clone());

        plan_write_file(&request, &mut plan, destination, "{}\n".to_string())
            .expect("write planning should succeed");

        assert!(plan.operations.is_empty());
    }

    #[test]
    fn whole_directory_backup_does_not_plan_child_backups() {
        let root = TestDir::new("whole-dir-backup");
        let source = root.path.join("source");
        let destination = root.path.join("destination");
        write_test_file(&source.join("file.txt"), "new");
        write_test_file(&destination.join("file.txt"), "old");
        let request = dry_run_request(vec![Category::Rules]);
        let mut plan = test_plan(destination.clone());

        copy_tree(
            &source,
            &destination,
            None,
            DirectoryConflictMode::WholeDirectory,
            &request,
            &mut plan,
        )
        .expect("copy tree planning should succeed");

        assert_eq!(
            plan.operations
                .iter()
                .filter(|operation| matches!(operation, InstallOperation::BackupPath { .. }))
                .count(),
            1
        );
        assert!(plan.operations.iter().any(|operation| {
            matches!(
                operation,
                InstallOperation::BackupPath { path, .. } if path == &destination
            )
        }));
        assert!(plan.operations.iter().all(|operation| {
            !matches!(
                operation,
                InstallOperation::BackupPath { path, .. } if path == &destination.join("file.txt")
            )
        }));
        assert!(plan.operations.iter().any(|operation| {
            matches!(
                operation,
                InstallOperation::CopyFile { destination: file_destination, .. }
                    if file_destination == &destination.join("file.txt")
            )
        }));
    }

    #[test]
    fn selective_directory_conflict_plans_file_conflicts() {
        let root = TestDir::new("selective-dir");
        let source = root.path.join("source");
        let destination = root.path.join("destination");
        write_test_file(&source.join("file.txt"), "new");
        write_test_file(&destination.join("file.txt"), "old");
        let mut request = dry_run_request(vec![Category::Rules]);
        request.conflict_mode = ConflictMode::Selective;
        let mut plan = test_plan(destination.clone());

        copy_tree(
            &source,
            &destination,
            None,
            DirectoryConflictMode::WholeDirectory,
            &request,
            &mut plan,
        )
        .expect("copy tree planning should succeed");

        assert!(plan.operations.iter().all(|operation| {
            !matches!(operation, InstallOperation::BackupPath { path, .. } if path == &destination)
        }));
        assert!(plan.operations.iter().any(|operation| {
            matches!(
                operation,
                InstallOperation::PromptConflict { target, .. }
                    if target == &destination.join("file.txt")
            )
        }));
        assert!(plan.operations.iter().any(|operation| {
            matches!(
                operation,
                InstallOperation::CopyFile { destination: file_destination, .. }
                    if file_destination == &destination.join("file.txt")
            )
        }));
    }

    #[test]
    fn overwrite_file_target_removes_existing_directory() {
        let root = TestDir::new("overwrite-directory");
        let source = root.path.join("source.txt");
        let destination = root.path.join("destination.txt");
        write_test_file(&source, "new");
        fs::create_dir_all(&destination).expect("directory target should be created");
        let mut request = dry_run_request(vec![Category::Rules]);
        request.conflict_mode = ConflictMode::Overwrite;
        let mut plan = test_plan(root.path.clone());

        plan_copy_file_with_state(&request, &mut plan, source, destination.clone(), false)
            .expect("copy planning should succeed");

        assert_eq!(plan.operations.len(), 2);
        assert!(matches!(
            &plan.operations[0],
            InstallOperation::RemovePath { path } if path == &destination
        ));
        assert!(matches!(
            &plan.operations[1],
            InstallOperation::CopyFile { destination: file_destination, .. }
                if file_destination == &destination
        ));
    }
}
