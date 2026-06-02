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
            execute_operation(operation)?;
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
            Category::Agents | Category::Hooks => {
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
    if request.stacks.is_empty() {
        return Ok(());
    }

    let destination_root = skills_root_for(plan.editor, plan.scope, &plan.target_root)?;
    for stack in selected_stack_names(request)? {
        if stack.is_empty() {
            continue;
        }
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

fn selected_stack_names(request: &InstallRequest) -> Result<Vec<String>> {
    if request.stacks.iter().any(|stack| stack == "all") {
        return available_stack_names(&request.root);
    }

    Ok(request.stacks.clone())
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

    plan_write_file(request, plan, destination, contents);
    Ok(())
}

fn selected_mcp_config(request: &InstallRequest) -> Result<Value> {
    let path = request.root.join(CONFIGS_DIR).join("mcps").join("mcp.json");
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let source: Value = serde_json::from_str(&contents)
        .with_context(|| format!("invalid MCP JSON {}", path.display()))?;
    if request.mcp_servers.is_empty() {
        return Ok(source);
    }

    let servers = source
        .get(MCP_SERVERS_KEY)
        .and_then(Value::as_object)
        .context("MCP config missing mcpServers object")?;
    let mut selected = Map::new();
    for key in &request.mcp_servers {
        if let Some(server) = servers.get(key) {
            selected.insert(key.clone(), server.clone());
        }
    }

    let mut root = Map::new();
    root.insert(MCP_SERVERS_KEY.to_string(), Value::Object(selected));
    Ok(Value::Object(root))
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

    if directory_conflict_mode == DirectoryConflictMode::WholeDirectory
        && is_nonempty_directory(destination)?
        && !plan_directory_conflict(request, plan, destination)
    {
        return Ok(());
    }

    for source_file in collect_source_files(source, exclude_prefix)? {
        let relative_path = source_file.strip_prefix(source).with_context(|| {
            format!(
                "failed to derive relative path for {}",
                source_file.display()
            )
        })?;
        let destination_file = destination.join(relative_path);
        plan_copy_file(request, plan, source_file, destination_file);
    }

    Ok(())
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

fn plan_copy_file(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    source: PathBuf,
    destination: PathBuf,
) {
    if destination.exists() && !plan_file_conflict(request, plan, &destination) {
        return;
    }
    plan.operations.push(InstallOperation::CopyFile {
        source,
        destination,
    });
}

fn plan_write_file(
    request: &InstallRequest,
    plan: &mut InstallPlan,
    destination: PathBuf,
    contents: String,
) {
    if destination.exists() && !plan_file_conflict(request, plan, &destination) {
        return;
    }
    plan.operations.push(InstallOperation::WriteFile {
        destination,
        contents,
    });
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
    if destination.exists() && request.conflict_mode == ConflictMode::Skip {
        plan.operations.push(InstallOperation::Skip {
            target: destination.to_path_buf(),
            reason: "conflict mode is skip".to_string(),
        });
        return Ok(());
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
        ConflictMode::Backup | ConflictMode::Selective => {
            plan.operations.push(InstallOperation::BackupPath {
                path: destination.to_path_buf(),
                backup: backup_path(destination, &plan_backup_timestamp(request)),
            });
            true
        }
        ConflictMode::Overwrite => true,
    }
}

fn execute_operation(operation: &InstallOperation) -> Result<()> {
    match operation {
        InstallOperation::CopyFile {
            source,
            destination,
        } => {
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
            ensure_parent(destination)?;
            fs::write(destination, contents)
                .with_context(|| format!("failed to write {}", destination.display()))?;
        }
        InstallOperation::AppendFile {
            destination,
            contents,
        } => {
            ensure_parent(destination)?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(destination)
                .with_context(|| format!("failed to open {}", destination.display()))?;
            file.write_all(contents.as_bytes())
                .with_context(|| format!("failed to append {}", destination.display()))?;
        }
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
    if request.yes && !request.dry_run && request.conflict_mode == ConflictMode::Selective {
        return ConflictMode::Backup;
    }
    request.conflict_mode
}

fn plan_backup_timestamp(request: &InstallRequest) -> String {
    if request.dry_run {
        "YYYYMMDDHHMMSS".to_string()
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

    fn dry_run_request(categories: Vec<Category>) -> InstallRequest {
        InstallRequest {
            root: PathBuf::from("."),
            editors: vec![Editor::Codex],
            scope: Scope::Project,
            categories,
            stacks: Vec::new(),
            mcp_servers: Vec::new(),
            conflict_mode: ConflictMode::Backup,
            yes: true,
            dry_run: true,
        }
    }

    #[test]
    fn filters_codex_project_mcp_category() {
        let request = dry_run_request(vec![Category::Skills, Category::Mcps]);
        let categories = filtered_categories(&request, Editor::Codex);
        assert_eq!(categories, vec![Category::Skills]);
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
}
