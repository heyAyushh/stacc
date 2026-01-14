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