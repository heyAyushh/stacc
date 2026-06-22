# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) and Cursor IDE (https://cursor.com) when working with code in this repository.

## What this repo is
- `stacc` is a set of **agent configuration files** (rules/commands/skills/agents/hooks) plus a Rust CLI/TUI (`stacc`) that copies them into tool-specific global or project folders.
- `install.sh` is a bootstrap and legacy-flag adapter. It installs or runs the Rust binary, then forwards installation work to `stacc install`, `stacc sync`, `stacc update`, or `stacc uninstall`.

## Common commands
- **Run installer (local checkout)**: `cargo run`
- **Run legacy bootstrap (local checkout)**: `./install.sh`
- **Dry-run manifest sync**: `cargo run -- sync --editor codex --scope project --dry-run --print-plan`
- **Dry-run managed update**: `cargo run -- update --editor codex --skill ultragoal --dry-run --print-plan`
- **Dry-run managed uninstall**: `cargo run -- uninstall --editor codex --skill ultragoal --dry-run --print-plan`
- **Run installer (remote / curl)**: `curl -fsSL https://raw.githubusercontent.com/heyAyushh/stacc/main/install.sh | bash`
- **Run full Rust check gate**: `cargo run -- check`
- **Validate installer syntax**: `bash -n install.sh`
- **Lint installer (recommended)**: `shellcheck -x install.sh`

## High-level structure
- `configs/`
  - `rules/` and `stack/*.mdc`: always-applied rules + optional stack-specific rule packs.
  - `commands/`: slash commands (markdown prompts).
  - `skills/`: skill folders; each contains a `SKILL.md` and optional references/scripts.
  - `agents/`: agent prompts used by some workflows.
  - `hooks/`: hook prompts/docs.
  - `mcps/`: MCP server configuration (`mcp.json`) and notes.
  - `codex-plugins/`: optional Codex plugin marketplace catalog (`plugins.json`).
- `.cursor/`: repo-local Cursor setup used for developing on this repo itself (commands/skills, etc.).
- `src/`: Rust CLI/TUI product. `src/install.rs` owns install planning/execution, conflict handling, rules summaries, and MCP config install/merge logic.
- `install.sh`: shell bootstrap and legacy-flag adapter for curl/local convenience.

## Installer behavior notes (important for edits)
- **macOS compatibility**: prefer Bash 3.2-safe patterns (no associative arrays).
- **MCP config**: `configs/mcps/mcp.json` is installed by Rust. JSON targets use recursive `serde_json` merge; Codex TOML targets use `toml_edit`; AMP settings are wrapped under `amp.mcpServers`.
- **Codex plugin catalog**: `configs/codex-plugins/plugins.json` is opt-in and Codex global only. Explicit `--codex-plugin` keys imply the internal category and global target; do not require users to pass `--category codex-plugins` or `--scope global`. Plan fixed `codex plugin marketplace add` and `codex plugin add` argv entries, and do not vendor plugin payloads into stacc.
- **Managed ownership manifest**: installs write `<target-root>/.stacc/manifest.json` for stacc-owned skills, stacks, command-as-skill installs, and Codex plugins. `stacc sync` backfills already-installed stacc skill folders by scanning known stacc source packages and only recording destinations that exist. `stacc update` and `stacc uninstall` must require matching manifest entries instead of scanning arbitrary editor folders. For Codex plugins, backfill is explicit with `--codex-plugin`; update through `codex plugin marketplace upgrade` and `codex plugin add`; uninstall through `codex plugin remove` and remove the marketplace only when no remaining stacc-managed plugin entry uses it.
- **Conflict resolution**: the Rust installer supports overwrite, backup (timestamped), skip, and selective per-file handling when targets already exist.

## Supported tools and target directories
The installer supports multiple AI coding tools with different target directory structures:

- **Cursor**: Global `~/.cursor/`, Project `.cursor/`
- **Claude Code**: Global `~/.claude/`, Project `.claude/`
- **Codex**: Global `~/.codex/`, Project `.codex/`
- **OpenCode**: Global `~/.config/opencode/`, Project `.opencode/`
- **AMP Code**: Global `~/.config/amp/`, Project `.agents/`

MCP configuration file locations vary by tool:
- **Claude**: `.mcp.json` (project) or `~/.claude.json` (global)
- **Cursor/Codex/OpenCode**: `mcp.json` in respective config directories
- **AMP Code**: `~/.config/amp/settings.json` → `amp.mcpServers` (OAuth in `~/.amp/oauth/`)

## MCP configuration merging
When installing MCP configs, the installer:
1. Reads `configs/mcps/mcp.json`
2. Filters selected MCP server keys when `--mcp-server` is provided
3. Recursively merges JSON object targets with destination first, then source
4. Renders Codex MCP servers into `[mcp_servers.<name>]` TOML tables with `toml_edit`
5. Applies the selected conflict mode before writing changed config files

## Conflict resolution modes
The installer provides several conflict resolution strategies:
- **Overwrite**: Replace existing files
- **Backup**: Create timestamped backups (`.bak.<timestamp>`)
- **Skip**: Leave existing files unchanged
- **Selective**: Resolve conflicts per file instead of replacing a whole directory

## Available skills
Skills in `configs/skills/` provide specialized capabilities:
- **karpathy-guidelines**: Behavioral guidelines to reduce common LLM coding mistakes.
- **bash-expert**: Bash/shell scripting help and debugging
- **changelog-generator**: Generate changelogs from git history
- **find-skills**: Discover and install agent skills
- **frontend-design**: Production-grade UI/frontend development
- **mcp-builder**: Guide for creating MCP servers
- **skill-creator**: Guide for creating new skills

## Available stacks
Stacks in `configs/stacks/` are framework/language-specific skill bundles:
- **bun**: Bun.js runtime and tooling
- **databases**: Database patterns (PostgreSQL, etc.)
- **ios**: SwiftUI, Swift concurrency, performance auditing, Liquid Glass
- **nextjs**: Next.js and React patterns
- **react-native**: React Native mobile development
- **rust**: Rust development patterns
- **solana**: Solana blockchain development
- **turborepo**: Monorepo tooling with Turborepo
- **typescript**: TypeScript conventions

## Development workflow
When adding or modifying configurations:
1. Edit files in `configs/` directory structure
2. Test locally with `cargo run -- install --dry-run --print-plan ...`
3. Validate with `cargo run -- check`
4. Validate bootstrap syntax with `bash -n install.sh` and lint with `shellcheck -x install.sh` when available
5. Ensure Bash 3.2 compatibility in `install.sh` (avoid associative arrays and Bash 4-only features)
6. For MCP changes: verify JSON validity and test merge behavior
7. For Codex plugin catalog changes: verify dry-run command planning and source/license attribution
8. Update README.md attributions if adapting from external sources

## Documentation-derived scripts
- Scripts that refresh, validate, summarize, or otherwise depend on external documentation are recurring maintenance workflows, not one-off scratch files.
- Put those scripts in the relevant tracked package, skill, stack, or config folder under `configs/` so every worktree gets the same workflow.
- Document the cadence, source URLs, expected outputs, and verification command next to the script.
- Keep generated outputs deterministic and commit any required lockfile/metadata updates with the script change.
- Do not leave documentation-derived scripts only in a local checkout, ignored folder, shell history, or untracked worktree.

## Adding new configurations

### New skill
1. Create `configs/skills/<skill-name>/SKILL.md` with frontmatter (name, description)
2. Add optional `references/` for context docs and `scripts/` for tooling
3. Update README.md attributions table if adapted from external source

### New stack
1. Create `configs/stacks/<stack-name>/` directory
2. Add `.mdc` rule files or skill folders with `SKILL.md`
3. Update README.md attributions table

### New rule
1. Add `.mdc` file to `configs/rules/` (always-applied) or `configs/stacks/<stack>/` (stack-specific)
2. Use Cursor MDC format with frontmatter if needed
