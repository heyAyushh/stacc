# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) and Cursor IDE (https://cursor.com) when working with code in this repository.

## What this repo is
- `stacc` is a set of **agent configuration files** (rules/commands/skills/agents/hooks) plus an **interactive installer** (`install.sh`) that copies them into tool-specific global or project folders.

## Common commands
- **Run installer (local checkout)**: `./install.sh`
- **Run installer (remote / curl)**: `curl -fsSL https://raw.githubusercontent.com/heyAyushh/stacc/main/install.sh | bash`
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
- `.cursor/`: repo-local Cursor setup used for developing on this repo itself (commands/skills, etc.).
- `install.sh`: the “product” — interactive TUI installer with conflict handling (force/backup/skip/selective) and MCP config install/merge logic.

## Installer behavior notes (important for edits)
- **macOS compatibility**: prefer Bash 3.2-safe patterns (no associative arrays).
- **MCP config**: `configs/mcps/mcp.json` is copied (or optionally merged when `jq` is available) into the target tool’s expected MCP file location.
- **Conflict resolution**: the installer supports overwrite, backup (timestamped), skip, and per-file selective decisions when targets already exist.

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
1. Checks if `jq` is available for JSON merging
2. If target exists and `jq` is present: prompts to merge (interactive) or merges automatically (non-interactive)
3. Merging uses `jq -s '.[0] * .[1]'` to deep-merge JSON objects (destination first, then source)
4. If `jq` is unavailable or merge declined: overwrites the target file (subject to conflict resolution)

## Conflict resolution modes
The installer provides several conflict resolution strategies:
- **Overwrite**: Replace existing files
- **Backup**: Create timestamped backups (`.bak.YYYYMMDDHHMMSS`)
- **Skip**: Leave existing files unchanged
- **Selective**: Per-file decision during installation

## Development workflow
When adding or modifying configurations:
1. Edit files in `configs/` directory structure
2. Test locally with `./install.sh` (dry-run available with flags)
3. Ensure Bash 3.2 compatibility (avoid associative arrays, use indexed arrays)
4. For MCP changes: verify JSON validity and test merge behavior
5. Update README.md attributions if adapting from external sources