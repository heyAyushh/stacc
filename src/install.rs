use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
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
const CODEX_CONFIG_FILE: &str = "config.toml";
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

#[derive(Clone, Debug)]
struct InstallContext {
    project_root: PathBuf,
    home: PathBuf,
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
        if self.categories.is_empty() {
            anyhow::bail!("install needs at least one category");
        }
        if !self.dry_run && !self.yes {
            anyhow::bail!("review dry-run first, then pass --yes for writes");
        }
        if !self.root.join(CONFIGS_DIR).is_dir() {
            anyhow::bail!("configs/ missing under {}", self.root.display());
        }
        Ok(())
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

    if plans.is_empty() {
        anyhow::bail!("selected categories are unsupported for selected editors/scope");
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
        .filter(|category| category.is_supported_for(editor, request.scope, &request.root))
        .collect()
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
    copy_tree(&source, &destination, None, mode, request, plan)
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
        copy_tree(
            &skills,
            &destination,
            None,
            DirectoryConflictMode::PerFile,
            request,
            plan,
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
        copy_tree(
            &source,
            &destination,
            None,
            DirectoryConflictMode::PerFile,
            request,
            plan,
        )?;
    }
    Ok(())
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
    }
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
        InstallOperation::Skip { .. } => {}
    }
    Ok(())
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
