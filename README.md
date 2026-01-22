# stacc
Curated configs for an AI coding workflow with muscles.
  
![Cursor](https://img.shields.io/badge/Cursor-black?style=flat&logo=cursor) ![Claude Code](https://img.shields.io/badge/Claude_Code-cc785c?style=flat&logo=anthropic) ![Codex](https://img.shields.io/badge/Codex-10a37f?style=flat&logo=openai&logoColor=white) ![OpenCode](https://img.shields.io/badge/OpenCode-1a1a2e?style=flat&logo=go&logoColor=00ADD8) ![AMP](https://img.shields.io/badge/AMP-ff5543?style=flat&logo=sourcegraph&logoColor=white)

![stacc banner](stacc.png)

## Installation

### Quick Install

```bash
curl -fsSL ay.dog | bash
```

**Or using GitHub URL:**
```bash
curl -fsSL https://raw.githubusercontent.com/heyAyushh/stacc/main/install.sh | bash
```

./### Local Install

```bash
git clone https://github.com/heyAyushh/stacc.git
cd stacc
./install.sh
```

The interactive installer will guide you through:
- **Editor selection**: Cursor, Claude Code, OpenCode, Codex, AMP Code
- **Scope selection**: Global (all projects) or project-specific
- **Category selection**: commands, rules, agents, skills, hooks, mcps
  - **Stack configs**: temporarily disabled in the installer UI 🤫

### Target Directories

| Scope | Codex | Claude Code | Cursor | Cursor Cloud Agents | OpenCode | AMP Code | GitHub (Copilot) | VS Code |
|-------|-------|--------|--------|---------------------|----------|----------|------------------|---------|
| Global | `~/.codex/` | `~/.claude/` | `~/.cursor/` | ❌ | `~/.config/opencode/` | `~/.config/amp/` | ❌ | ❌ |
| Project | `.codex/` | `.claude/` | `.cursor/` | ❌ | `.opencode/` | `.agents/` | ❌ | ❌ |

#### Configuration File Locations

#### Global Configuration File Locations (macOS/Linux)

| Config Type | Codex | Claude Code | Cursor | Cursor Cloud Agents | OpenCode | AMP Code | GitHub (Copilot) | VS Code |
|-------------|-------|--------|--------|---------------------|----------|----------|------------------|---------|
| LSPs | [#8745](https://github.com/openai/codex/issues/8745) ❌ | [plugin](https://code.claude.com/docs/en/plugins-reference#lsp-servers) | built-in ❌ | ❌ | built-in ❌ | built-in ❌ | built-in ❌ | extensions ❌ |
| Hooks | [#2109](https://github.com/openai/codex/issues/2109) ❌ | `~/.claude/settings.json` | `~/.cursor/hooks.json` | ❌ | ❌ | ❌ | ❌ | ❌ |
| Rules | `~/.codex/rules/*.rules`, `~/.codex/AGENTS.md` | `~/.claude/CLAUDE.md` | `~/.cursor/rules/`, `~/.cursor/AGENTS.md` | ❌ | `~/.config/opencode/AGENTS.md` | `~/.config/amp/AGENTS.md` | ❌ | ❌ |
| Skills | `~/.codex/skills/` | `~/.claude/skills/` | `~/.cursor/skills/` | ❌ | `~/.config/opencode/skills/` | `~/.config/agents/skills/` | ❌ | ❌ |
| Subagents | [#2604](https://github.com/openai/codex/issues/2604) ❌ | `~/.claude/agents/` | `~/.cursor/agents/` | ❌ | `~/.config/opencode/agents/` | built-in ❌ | ❌ | ❌ |
| MCPs (Model Context Protocol) | `~/.codex/config.toml` | `~/.claude.json` | `~/.cursor/mcp.json` | cursor global ❌ | `~/.config/opencode/.opencode.json` | `~/.config/amp/settings.json` | ❌ | ❌ |
| Commands | `~/.codex/skills/commands/` | `~/.claude/commands/` | `~/.cursor/commands/` | ❌ | `~/.config/opencode/commands/` | `~/.config/amp/commands/` | ❌ | ❌ |

#### Project-Specific Configuration File Locations (macOS/Linux)

| Config Type | Codex | Claude | Cursor | Cursor Cloud Agents | OpenCode | AMP Code | GitHub (Copilot) | VS Code |
|-------------|-------|--------|--------|---------------------|----------|----------|------------------|---------|
| LSPs | built-in ❌ | [plugin](https://code.claude.com/docs/en/plugins-reference#lsp-servers) | built-in ❌ | ❌ | built-in ❌ | built-in ❌ | ❌ | extensions ❌ |
| Hooks | ❌ | `.claude/settings.json`, `.claude/settings.local.json` | `.cursor/hooks.json` | ❌ | ❌ | ❌ | ❌ | ❌ |
| Rules | `AGENTS.md` | `CLAUDE.md` | `.cursor/rules/`, `AGENTS.md` | ❌ | `AGENTS.md` | `AGENTS.md` | `.github/copilot-instructions.md` | `.vscode/settings.json` |
| Skills | `.codex/skills/` | `.claude/skills/` | `.cursor/skills/` | ❌ | `.opencode/skills/` | `.agents/skills/` | ❌ | ❌ |
| Subagents | ❌ | `.claude/agents/` | `.cursor/agents/` | ❌ | `.opencode/agents/` | built-in ❌ | `.github/copilot-instructions.md` | ❌ |
| MCPs | global ❌ | `.mcp.json` | `.cursor/mcp.json` | cursor global ❌ | `.opencode.json` | built-in ❌ | ❌ | ❌ |
| Commands | `.codex/skills/commands/` | `.claude/commands/` | `.cursor/commands/` | ❌ | `.opencode/commands/` | `.agents/commands/` | ❌ | `<project>/.vscode/tasks.json` |

**Notes / Exceptions:**
* Codex tracking: LSP [#8745](https://github.com/openai/codex/issues/8745), Hooks [#2109](https://github.com/openai/codex/issues/2109), Subagents [#2604](https://github.com/openai/codex/issues/2604)
* Cursor Cloud Agents: uses Cursor global config only
* OpenCode MCPs: `~/.config/opencode/.opencode.json` → `mcpServers`
* AMP MCPs: `~/.config/amp/settings.json` → `amp.mcpServers` (OAuth in `~/.amp/oauth/`)
* Codex commands: stored under `skills/commands/` for Codex installations
* VS Code LSP/config: extensions or settings
* Copilot: no user-defined MCPs/skills/commands
* VS Code user settings: macOS `~/Library/Application Support/Code/User/settings.json`, Linux `~/.config/Code/User/settings.json`
* Project root: `.vscode/`, `.github/`, `.codex/`, `.claude/`, `.cursor/`, `.opencode/`, `.agents/`
* Cursor rules vs skills: `.cursor/rules/` (apply modes), `.cursor/skills/` (agent-decided)
    

## Structure

```
configs/
├── agents/          # Agent definitions (verifier, askuserquestion)
├── commands/        # Slash commands (commit, deslop, ultrathink, etc.)
├── hooks/           # Git hooks
├── mcps/            # MCP server configurations
├── rules/           # Always-applied rules (clean-code, commit format, etc.)
├── skills/          # Modular skills for specific tasks
│   ├── changelog-generator/
│   ├── frontend-design/
│   ├── mcp-builder/
│   └── skill-creator/
└── stack/           # Language/framework-specific configurations
    ├── ios-skills/  # SwiftUI and iOS development skills
    └── *.mdc        # Stack-specific rules (bun, typescript, etc.)
```

## Attributions

This repository contains configurations adapted from open-source projects. Below are the attributions for code copied or adapted from external sources.

### From [anthropics/skills](https://github.com/anthropics/skills) (Apache-2.0)

| File | Description |
|------|-------------|
| `configs/skills/mcp-builder/` | MCP Server Development Guide - creating high-quality MCP servers |
| `configs/skills/skill-creator/` | Skill Creator Guide - creating effective Claude skills |
| `configs/skills/frontend-design/` | Frontend Design - distinctive, production-grade UI creation |

### From [Dimillian/Skills](https://github.com/Dimillian/Skills)

| File | Description |
|------|-------------|
| `configs/stack/ios-skills/swift-concurrency-expert/` | Swift 6.2+ concurrency review and remediation |
| `configs/stack/ios-skills/swiftui-view-refactor/` | SwiftUI view refactoring patterns |
| `configs/stack/ios-skills/swiftui-performance-audit/` | SwiftUI performance auditing and optimization |
| `configs/stack/ios-skills/swiftui-ui-patterns/` | SwiftUI UI patterns and best practices |
| `configs/stack/ios-skills/swiftui-liquid-glass/` | iOS 26+ Liquid Glass API implementation |
| `configs/stack/ios-skills/ios-debugger-agent/` | XcodeBuildMCP-based iOS debugging |

### From [triggerdotdev/trigger.dev](https://github.com/triggerdotdev/trigger.dev) and community (Apache-2.0)

| File | Description |
|------|-------------|
| `configs/commands/deslop.md` | Remove AI-generated code slop |

The `deslop.md` command appears in multiple repositories including:
- [triggerdotdev/trigger.dev](https://github.com/triggerdotdev/trigger.dev) (Apache-2.0)
- [fatih/dotfiles](https://github.com/fatih/dotfiles)
- [moeru-ai/airi](https://github.com/moeru-ai/airi) (MIT)

### From Claude Code / Anthropic System Prompts

| File | Description |
|------|-------------|
| `configs/agents/askuserquestion.md` | AskUserQuestion tool description |
| `configs/commands/explore.md` | File search specialist agent prompt |

These are adaptations of Claude Code's built-in tool descriptions and agent prompts. Also documented in:
- [Piebald-AI/claude-code-system-prompts](https://github.com/Piebald-AI/claude-code-system-prompts) (MIT)

### From [sanjeed5/awesome-cursor-rules-mdc](https://github.com/sanjeed5/awesome-cursor-rules-mdc) (CC0-1.0)

| File | Description |
|------|-------------|
| `configs/stack/bun.mdc` | Bun.js best practices |
| `configs/stack/typescript.mdc` | TypeScript conventions |
| `configs/stack/postgresql.mdc` | PostgreSQL guidelines |

### From [PatrickJS/awesome-cursorrules](https://github.com/PatrickJS/awesome-cursorrules) (CC0-1.0)

| File | Description |
|------|-------------|
| `configs/rules/clean-code.mdc` | Clean code guidelines |

### From [hamzafer/cursor-commands](https://github.com/hamzafer/cursor-commands) (MIT)

| File | Description |
|------|-------------|
| `configs/commands/rebase.md` | Rebase the current branch to resolve/maybe Merge Conflicts |

### From [Raine Virta - blog](https://raine.dev/blog/resolve-conflicts-with-claude)

| File | Description |
|------|-------------|
| `configs/commands/clean-gone.md` | Cleans up all git branches marked as [gone] (branches that have been deleted on the remote but still exist locally), including removing associated worktrees. |
| `configs/commands/review-pr.md` | Review Pull request from github |

### From [anthropics/claude-code](https://github.com/anthropics/claude-code/blob/main/plugins/code-review/commands/code-review.md) [License](https://github.com/anthropics/claude-code/blob/main/LICENSE.md)

| File | Description |
|------|-------------|
| `configs/commands/visualize.md` | Mermaid diagram generation |
| `configs/commands/onboard-new-developer.md` | Developer onboarding checklist |
| `configs/commands/refactor.md` | Code refactoring checklist (refactor-code.md) |

### From [kinopeee/cursorrules](https://github.com/kinopeee/cursorrules) (MIT)

| File | Description |
|------|-------------|
| `configs/commands/commit.md` | Git commit workflow (commit-only.md) |
| `configs/commands/commit-push.md` | Commit and push workflow |
| `configs/commands/commit-push-pr.md` | Commit, push, and PR workflow |
| `configs/rules/commit-message-format.mdc` | Conventional Commits format |
| `configs/rules/pr-message-format.mdc` | PR message format |
| `configs/rules/prompt-injection-gaurd.mdc` | External context injection defense (prompt-injection-guard.mdc) |

### From [anthropics/claude-code-security-review](https://github.com/anthropics/claude-code-security-review) (MIT)

| File | Description |
|------|-------------|
| `configs/commands/review.md` | Security-focused code review |

### From [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills)

| File | Description |
|------|-------------|
| `configs/commands/council.md` | Spawn multiple agents to deeply explore a codebase area before acting |
| `configs/commands/iterate-browser.md` | Autonomously iterate on UI changes using console.log and browser tools |

### From [@shaoruu](https://shaoruu.io/cursor/council)

| File | Description |
|------|-------------|
| `configs/skills/changelog-generator/` | Changelog generation from git commits |

Also found in:
- [davila7/claude-code-templates](https://github.com/davila7/claude-code-templates) (MIT)
- [skillcreatorai/Ai-Agent-Skills](https://github.com/skillcreatorai/Ai-Agent-Skills) (MIT)

### Original / Sources Not Found

| File | Description |
|------|-------------|
| `configs/commands/ultrathink.md` | Deep reasoning mode protocol (original) |
| `configs/commands/init.md` | AGENTS.md initialization |
| `configs/agents/verifier.md` | Work verification agent |

## License

MIT. [LICENSE](LICENSE).

Individual components retain their original licenses:
- Anthropic skills: Apache-2.0 (see `LICENSE.txt` in skill directories)
- Dimillian/Skills: Check repository for license
- Other components: See individual source repositories
