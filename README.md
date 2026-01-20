# stacc

A collection of AI agent configurations, skills, commands, and rules for Claude Code and Cursor IDE.

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
- **Editor selection**: Cursor, Claude Code, or both
- **Scope selection**: Global (all projects) or project-specific
- **Category selection**: commands, rules, agents, skills, stack configs

### Manual Installation

Copy the desired configurations to your project's `.cursor/` or `.claude/` directory:

```bash
# Example: Copy commands to Cursor
cp -r configs/commands/ .cursor/commands/

# Example: Copy rules to Claude Code globally
cp -r configs/rules/ ~/.claude/rules/
```

### Target Directories

| Scope | Cursor | Claude Code |
|-------|--------|-------------|
| Global | `~/.cursor/` | `~/.claude/` |
| Project | `.cursor/` | `.claude/` |

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
| `configs/commands/clean-code.mdc` | Clean code guidelines |

### From [Raine Virta - blog](https://raine.dev/blog/resolve-conflicts-with-claude)

| File | Description |
|------|-------------|
| `configs/commands/review-pr.mdc` | Review Pull request from github |

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

Individual components retain their original licenses:
- Anthropic skills: Apache-2.0 (see `LICENSE.txt` in skill directories)
- Dimillian/Skills: Check repository for license
- Other components: See individual source repositories

