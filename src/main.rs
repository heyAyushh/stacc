mod bootstrap;
mod bundle;
mod catalog;
mod check;
mod config;
mod git_utils;
mod hook_selection;
mod install;
mod metadata;
mod panel;
mod selective;

use std::io::{self, IsTerminal};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use bootstrap::{default_bootstrap_options, run_bootstrap};
use bundle::resolve_runtime_root;
use catalog::{discover_catalog, Category, ConflictMode, Editor, Scope};
use check::{run_checks, CheckOptions};
use config::{load_panel_config, PanelConfig};
use install::{
    build_install_plan, build_manage_plan, build_sync_manifest_plan, execute_install_request,
    execute_manage_request, execute_sync_manifest_request, print_plan, InstallRequest,
    ManageAction, ManageRequest, SyncManifestRequest,
};
use metadata::{default_sync_options, sync_metadata};
use panel::{run_panel, PanelOutcome};

#[derive(Debug, Parser)]
#[command(name = "stacc")]
#[command(about = "Rust control panel for stacc configs")]
#[command(version)]
#[command(after_long_help = TOP_LEVEL_EXAMPLES)]
struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Use a checked-out stacc repo instead of bundled configs"
    )]
    root: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        value_name = "PATH",
        help = "Load panel defaults from this JSON config"
    )]
    config: Option<PathBuf>,
    #[arg(long, global = true, help = "Open the TUI control panel")]
    panel: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print repository/catalog status.
    Status(StatusArgs),
    /// Run the installer through the typed Rust control layer.
    Install(InstallArgs),
    #[command(about = "Update stacc-managed skills or Codex plugins")]
    Update(ManageArgs),
    #[command(about = "Uninstall stacc-managed skills or Codex plugins")]
    Uninstall(ManageArgs),
    #[command(about = "Sync stacc ownership for already-installed skills or plugins")]
    Sync(SyncArgs),
    /// Sync generated skill license/version/origin metadata.
    SyncMetadata(SyncMetadataArgs),
    /// Install or upgrade the stacc binary from GitHub.
    Bootstrap(BootstrapArgs),
    /// Run repository checks and installed-binary smoke tests.
    Check(CheckArgs),
}

#[derive(Debug, Args)]
#[command(after_long_help = STATUS_EXAMPLES)]
struct StatusArgs {
    #[arg(long, help = "Print machine-readable JSON")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(after_long_help = INSTALL_EXAMPLES)]
struct InstallArgs {
    #[arg(long = "editor", value_enum, required = true, help = "Target editor")]
    editors: Vec<Editor>,
    #[arg(long, value_enum, default_value_t = Scope::Project, help = "Install scope")]
    scope: Scope,
    #[arg(
        long = "category",
        value_enum,
        help = "Config category to install; optional when --codex-plugin is set"
    )]
    categories: Vec<Category>,
    #[arg(long = "stack", help = "Stack skill folder to install")]
    stacks: Vec<String>,
    #[arg(long = "mcp-server", help = "MCP server key to install")]
    mcp_servers: Vec<String>,
    #[arg(long = "hook", help = "Hook package to install")]
    hook_packages: Vec<String>,
    #[arg(
        long = "codex-plugin",
        help = "Repeatable Codex marketplace plugin key"
    )]
    codex_plugins: Vec<String>,
    #[arg(long, value_enum, default_value_t = ConflictMode::Backup, help = "Conflict strategy")]
    conflict: ConflictMode,
    #[arg(long, help = "Allow writes without interactive confirmation")]
    yes: bool,
    #[arg(long, help = "Print planned operations without writing")]
    dry_run: bool,
    #[arg(long, help = "Print planned install operations")]
    print_plan: bool,
}

#[derive(Debug, Args)]
#[command(after_long_help = MANAGE_EXAMPLES)]
struct ManageArgs {
    #[arg(long = "editor", value_enum, required = true, help = "Target editor")]
    editors: Vec<Editor>,
    #[arg(long, value_enum, default_value_t = Scope::Project, help = "Managed entry scope")]
    scope: Scope,
    #[arg(
        long = "skill",
        help = "Repeatable stacc-managed skill or stack folder name"
    )]
    skills: Vec<String>,
    #[arg(
        long = "codex-plugin",
        help = "Repeatable stacc-managed Codex plugin key"
    )]
    codex_plugins: Vec<String>,
    #[arg(long, value_enum, default_value_t = ConflictMode::Backup, help = "Conflict strategy")]
    conflict: ConflictMode,
    #[arg(long, help = "Allow writes without interactive confirmation")]
    yes: bool,
    #[arg(long, help = "Print planned operations without writing")]
    dry_run: bool,
    #[arg(long, help = "Print planned managed operations")]
    print_plan: bool,
}

#[derive(Debug, Args)]
#[command(after_long_help = SYNC_EXAMPLES)]
struct SyncArgs {
    #[arg(long = "editor", value_enum, required = true, help = "Target editor")]
    editors: Vec<Editor>,
    #[arg(long, value_enum, default_value_t = Scope::Project, help = "Managed entry scope")]
    scope: Scope,
    #[arg(
        long = "skill",
        help = "Repeatable installed stacc skill or stack folder name"
    )]
    skills: Vec<String>,
    #[arg(
        long = "codex-plugin",
        help = "Repeatable stacc Codex plugin key to mark as managed"
    )]
    codex_plugins: Vec<String>,
    #[arg(long, help = "Allow manifest writes")]
    yes: bool,
    #[arg(long, help = "Print planned manifest updates without writing")]
    dry_run: bool,
    #[arg(long, help = "Print planned manifest sync operations")]
    print_plan: bool,
}

#[derive(Debug, Args)]
#[command(after_long_help = SYNC_METADATA_EXAMPLES)]
struct SyncMetadataArgs {
    #[arg(long, value_name = "PATH", help = "Write lockfile to this path")]
    output: Option<PathBuf>,
    #[arg(long, help = "Refresh current GitHub origin HEAD commits")]
    refresh_origin: bool,
    #[arg(long, help = "Audit without writing the lockfile")]
    dry_run: bool,
    #[arg(long, help = "Print machine-readable JSON report")]
    json: bool,
}

#[derive(Debug, Args)]
#[command(after_long_help = BOOTSTRAP_EXAMPLES)]
struct BootstrapArgs {
    #[arg(long, value_name = "URL", help = "GitHub repo URL to install from")]
    repo_url: Option<String>,
    #[arg(long, help = "Print the cargo install command without running it")]
    dry_run: bool,
}

#[derive(Debug, Args)]
#[command(after_long_help = CHECK_EXAMPLES)]
struct CheckArgs {
    #[arg(long, help = "Fail when shellcheck is not installed")]
    require_shellcheck: bool,
}

const TOP_LEVEL_EXAMPLES: &str = "\
Examples:
  stacc
  stacc --panel
  stacc status
  stacc install --editor codex --scope global --category rules --category skills --dry-run
  stacc sync --editor codex --scope project --dry-run --print-plan
  stacc update --editor codex --skill ultragoal --dry-run --print-plan
  stacc uninstall --editor codex --codex-plugin lazycodex --dry-run --print-plan
  stacc bootstrap --dry-run
  stacc check
";

const STATUS_EXAMPLES: &str = "\
Examples:
  stacc status
  stacc status --json
";

const INSTALL_EXAMPLES: &str = "\
Examples:
  stacc install --editor cursor --scope project --category rules --category skills --dry-run
  stacc install --editor codex --scope global --category rules --category skills --category mcps --mcp-server github --yes
  stacc install --editor cursor --scope project --category hooks --hook continual-learning --dry-run
  stacc install --editor cursor --scope project --category cursor-plugins --yes
  stacc install --editor codex --codex-plugin lazycodex --dry-run --print-plan
";

const MANAGE_EXAMPLES: &str = "\
Examples:
  stacc update --editor codex --skill ultragoal --dry-run --print-plan
  stacc update --editor codex --codex-plugin lazycodex --dry-run --print-plan
  stacc uninstall --editor codex --skill ultragoal --dry-run --print-plan
  stacc uninstall --editor codex --codex-plugin lazycodex --dry-run --print-plan
";

const SYNC_EXAMPLES: &str = "\
Examples:
  stacc sync --editor codex --scope project --dry-run --print-plan
  stacc sync --editor codex --scope project --skill ultragoal --dry-run --print-plan
  stacc sync --editor codex --codex-plugin lazycodex --dry-run --print-plan
";

const SYNC_METADATA_EXAMPLES: &str = "\
Examples:
  stacc sync-metadata --dry-run --json
  stacc sync-metadata --refresh-origin
";

const BOOTSTRAP_EXAMPLES: &str = "\
Examples:
  stacc bootstrap --dry-run
  stacc bootstrap
";

const CHECK_EXAMPLES: &str = "\
Examples:
  stacc check
  stacc check --require-shellcheck
";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = resolve_runtime_root(cli.root)?;

    if cli.panel && cli.command.is_some() {
        anyhow::bail!("--panel cannot be combined with a subcommand");
    }

    match cli.command {
        None => run_panel_command(root, cli.config),
        Some(Command::Status(args)) => run_status_command(root, args),
        Some(Command::Install(args)) => run_install_command(root, args),
        Some(Command::Update(args)) => run_manage_command(root, args, ManageAction::Update),
        Some(Command::Uninstall(args)) => run_manage_command(root, args, ManageAction::Uninstall),
        Some(Command::Sync(args)) => run_sync_command(root, args),
        Some(Command::SyncMetadata(args)) => run_sync_metadata_command(root, args),
        Some(Command::Bootstrap(args)) => run_bootstrap_command(args),
        Some(Command::Check(args)) => run_check_command(root, args),
    }
}

fn run_panel_command(root: PathBuf, config_path: Option<PathBuf>) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        anyhow::bail!(
            "no interactive terminal available; use `stacc install ... --dry-run` for automation or `stacc status --json` for machine output"
        );
    }

    let mut config = load_panel_config(&root, config_path)?;
    let mut message = None;
    loop {
        let catalog = discover_catalog(&root)?;
        match run_panel(root.clone(), catalog, config.clone(), message.take())? {
            PanelOutcome::Quit => break,
            PanelOutcome::RunInstall(request) => {
                config = PanelConfig::from_install_request(&request);
                let plan = build_install_plan(&request)?;
                print_plan(&plan);
                if request.dry_run {
                    message = Some("install dry-run complete".to_string());
                    continue;
                }
                let results = execute_install_request(&request)?;
                let operations = results
                    .iter()
                    .map(|result| result.operation_count)
                    .sum::<usize>();
                message = Some(format!(
                    "install complete: {} target(s), {} operation(s)",
                    results.len(),
                    operations
                ));
            }
            PanelOutcome::SyncMetadata(options) => {
                let report = sync_metadata(&options)?;
                message = Some(format!(
                    "metadata synced: {} skills, {} missing license, {} missing version, {} origin errors",
                    report.skill_count,
                    report.missing_license_count,
                    report.missing_version_count,
                    report.origin_error_count
                ));
            }
            PanelOutcome::RunChecks => {
                run_checks(&CheckOptions {
                    root: root.clone(),
                    require_shellcheck: false,
                })?;
                message = Some("checks passed".to_string());
            }
            PanelOutcome::Bootstrap(options) => {
                run_bootstrap(&options)?;
                message = Some(if options.dry_run {
                    "bootstrap dry-run complete".to_string()
                } else {
                    "bootstrap complete".to_string()
                });
            }
        }
    }
    Ok(())
}

fn run_status_command(root: PathBuf, args: StatusArgs) -> Result<()> {
    let catalog = discover_catalog(&root)?;
    let status = git_utils::repository_status(&root, &catalog::default_metadata_path(&root))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&(status, catalog))?);
    } else {
        println!("branch: {} @ {}", status.branch, status.head);
        println!("changed paths: {}", status.changed_paths);
        println!("skills: {}", catalog.skill_count);
        println!("stacks: {}", catalog.stacks.len());
        println!("mcp servers: {}", catalog.mcp_servers.join(","));
        println!("hooks: {}", catalog.hook_packages.len());
        println!("codex plugins: {}", catalog.codex_plugins.join(","));
        println!(
            "metadata lock: {}",
            catalog::default_metadata_path(&root).display()
        );
    }
    Ok(())
}

fn run_install_command(root: PathBuf, args: InstallArgs) -> Result<()> {
    let request = InstallRequest {
        root,
        editors: args.editors,
        scope: args.scope,
        categories: args.categories,
        stacks: args.stacks,
        mcp_servers: args.mcp_servers,
        hook_packages: args.hook_packages,
        codex_plugins: args.codex_plugins,
        conflict_mode: args.conflict,
        yes: args.yes,
        dry_run: args.dry_run,
    };
    let plan = build_install_plan(&request)?;
    if args.print_plan || request.dry_run {
        print_plan(&plan);
    }
    if !request.dry_run {
        execute_install_request(&request)?;
    }
    Ok(())
}

fn run_manage_command(root: PathBuf, args: ManageArgs, action: ManageAction) -> Result<()> {
    let request = ManageRequest {
        root,
        editors: args.editors,
        scope: args.scope,
        skills: args.skills,
        codex_plugins: args.codex_plugins,
        conflict_mode: args.conflict,
        yes: args.yes,
        dry_run: args.dry_run,
    };
    let plan = build_manage_plan(&request, action)?;
    if args.print_plan || request.dry_run {
        print_plan(&plan);
    }
    if !request.dry_run {
        execute_manage_request(&request, action)?;
    }
    Ok(())
}

fn run_sync_command(root: PathBuf, args: SyncArgs) -> Result<()> {
    let request = SyncManifestRequest {
        root,
        editors: args.editors,
        scope: args.scope,
        skills: args.skills,
        codex_plugins: args.codex_plugins,
        yes: args.yes,
        dry_run: args.dry_run,
    };
    let plan = build_sync_manifest_plan(&request)?;
    if args.print_plan || request.dry_run {
        print_plan(&plan);
    }
    if !request.dry_run {
        execute_sync_manifest_request(&request)?;
    }
    Ok(())
}

fn run_sync_metadata_command(root: PathBuf, args: SyncMetadataArgs) -> Result<()> {
    let mut options = default_sync_options(root.clone());
    if let Some(output) = args.output {
        options.output = output;
    }
    options.refresh_origin = args.refresh_origin;
    options.dry_run = args.dry_run;

    let report = sync_metadata(&options)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("skills: {}", report.skill_count);
        println!("missing license: {}", report.missing_license_count);
        println!("missing version: {}", report.missing_version_count);
        println!("origin errors: {}", report.origin_error_count);
        println!("output: {}", report.output.display());
    }
    Ok(())
}

fn run_bootstrap_command(args: BootstrapArgs) -> Result<()> {
    let mut options = default_bootstrap_options();
    if let Some(repo_url) = args.repo_url {
        options.repo_url = repo_url;
    }
    options.dry_run = args.dry_run;
    run_bootstrap(&options)
}

fn run_check_command(root: PathBuf, args: CheckArgs) -> Result<()> {
    run_checks(&CheckOptions {
        root,
        require_shellcheck: args.require_shellcheck,
    })
}
