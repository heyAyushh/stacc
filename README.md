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

### Local Install

```bash
git clone https://github.com/heyAyushh/stacc.git
cd stacc
./install.sh
```

The Rust control panel will guide you through:
- **Editor selection**: Cursor, Claude Code, OpenCode, Codex, AMP Code
- **Scope selection**: Global (all projects) or project-specific
- **Category selection**: commands, rules, agents, skills, stack, hooks, mcps, cursor-plugins, codex-skills
- **Stack selection**: choose one or more stack skill folders from `configs/stacks/`
- **Hook and MCP selection**: choose hook packages and MCP servers from the Hooks/MCP segment
- **Version actions**: refresh git status, sync skill metadata, run checks, or bootstrap/upgrade the binary

#### Stacks

Stacks are framework/language-specific skill bundles under `configs/stacks/`. When you select the `stack` category, the installer prompts you to choose one or more stack folders and installs them into each editor's `skills/` directory.

#### Installer options

`install.sh` is now a bootstrap and legacy-flag adapter. These commands install or run `stacc`, then forward to the Rust install engine:

```bash
./install.sh --categories commands,rules,skills,stack --stacks bun,typescript
```

```bash
./install.sh --all --project --categories skills,stack --stacks all
```

```bash
./install.sh --cursor --global --categories mcps --mcp-servers github,grep
```

```bash
./install.sh --cursor --project --categories hooks --hooks continual-learning
```

```bash
./install.sh --cursor --project --categories cursor-plugins
```

```bash
./install.sh --codex --project --categories codex-skills
```

### Rust Control Panel

The Rust TUI gives stacc a thin control panel for installation, customization, version metadata, hooks, and MCP selection.

Install the binary directly:

```bash
cargo install --git https://github.com/heyAyushh/stacc --locked --force
```

For a local checkout:

```bash
cargo install --path . --locked --force
```

The installed binary bundles `install.sh`, `configs/`, and README metadata, so agents do not need to pass `--root` after `cargo install`. It materializes the bundle into a versioned cache path on first run. Set `STACC_BUNDLE_ROOT=/path/to/cache` when an agent needs a deterministic bundle directory.

```bash
cargo run
```

Explicit flag form also works:

```bash
cargo run -- --panel
```

Useful non-interactive commands:

```bash
cargo run -- status
cargo run -- install --editor cursor --editor codex --scope project --category rules --category skills --dry-run --print-plan
cargo run -- install --editor cursor --scope project --category hooks --hook continual-learning --dry-run --print-plan
cargo run -- sync-metadata --refresh-origin
cargo run -- bootstrap --dry-run
cargo run -- check
```

After `cargo install`, drop `cargo run --`:

```bash
stacc
stacc install --editor codex --scope global --category rules --category skills --category mcps --mcp-server github --dry-run
stacc install --editor codex --scope global --category rules --category skills --category mcps --mcp-server github --yes
stacc install --editor cursor --scope project --category hooks --hook continual-learning --dry-run
stacc bootstrap --dry-run
stacc check
```

- Panel segments: Install, Customise, Version, Skills, Hooks/MCP
- Panel actions return to the TUI with a result message after install dry-runs, installs, metadata sync, checks, and bootstrap dry-runs
- Install execution is native Rust: file copying, conflict handling, rules summaries, hook package filtering, MCP JSON/TOML merge, and installed-binary smoke checks use typed Rust planning with explicit dry-run/yes gates
- `selective` conflict mode shows prompt operations in dry-run plans and prompts per conflicting file in an interactive terminal; non-interactive agents should use `backup`, `overwrite`, `skip`, or `--dry-run`
- Metadata sync writes `configs/metadata/skills.lock.json` with each skill's local path, license, version, source URL, declared origin commit, and current origin HEAD commit when GitHub lookup is enabled
- `stacc bootstrap` matches the shell bootstrap path by running `cargo install --git https://github.com/heyAyushh/stacc.git --locked --force`; `--dry-run` prints the command without network access
- Custom defaults live in `configs/stacc-panel.json`

### Checks

Run the full local gate before pushing installer or control-panel changes:

```bash
stacc check
```

The gate runs Rust format checks, tests, clippy, installer syntax checks, MCP/panel/skill metadata JSON validation, `cargo install --path`, and smoke checks against the installed `stacc` binary. It also runs `shellcheck -x install.sh` when `shellcheck` is installed. Use `stacc check --require-shellcheck` when CI must fail if `shellcheck` is missing.

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
| Rules | `~/.codex/AGENTS.md` | `~/.claude/CLAUDE.md` | `~/.cursor/rules/`, `~/.cursor/AGENTS.md` | ❌ | `~/.config/opencode/AGENTS.md` | `~/.config/amp/AGENTS.md` | ❌ | ❌ |
| Skills | `~/.codex/skills/` | `~/.claude/skills/` | `~/.cursor/skills/` | ❌ | `~/.config/opencode/skills/` | `~/.config/agents/skills/` | ❌ | ❌ |
| Subagents | [#2604](https://github.com/openai/codex/issues/2604) ❌ | `~/.claude/agents/` | `~/.cursor/agents/` | ❌ | `~/.config/opencode/agents/` | built-in ❌ | ❌ | ❌ |
| MCPs (Model Context Protocol) | `~/.codex/config.toml` | `~/.claude.json` | `~/.cursor/mcp.json` | cursor global ❌ | `~/.config/opencode/.opencode.json` | `~/.config/amp/settings.json` | ❌ | ❌ |
| Commands | Migrated to skills `~/.codex/skills/` | Migrated to skills `~/.claude/skills/` | `~/.cursor/commands/` | ❌ | `~/.config/opencode/commands/` | Migrated to skills `~/.config/agents/skills/` | ❌ | ❌ |

#### Project-Specific Configuration File Locations (macOS/Linux)

| Config Type | Codex | Claude | Cursor | Cursor Cloud Agents | OpenCode | AMP Code | GitHub (Copilot) | VS Code |
|-------------|-------|--------|--------|---------------------|----------|----------|------------------|---------|
| LSPs | built-in ❌ | [plugin](https://code.claude.com/docs/en/plugins-reference#lsp-servers) | built-in ❌ | ❌ | built-in ❌ | built-in ❌ | ❌ | extensions ❌ |
| Hooks | ❌ | `.claude/settings.json`, `.claude/settings.local.json` | `.cursor/hooks.json` | ❌ | ❌ | ❌ | ❌ | ❌ |
| Rules | `AGENTS.md` | `CLAUDE.md` | `.cursor/rules/`, `AGENTS.md` | ❌ | `AGENTS.md` | `AGENTS.md` | `.github/copilot-instructions.md` | `.vscode/settings.json` |
| Skills | `.codex/skills/` | `.claude/skills/` | `.cursor/skills/` | ❌ | `.opencode/skills/` | `.agents/skills/` | ❌ | ❌ |
| Subagents | ❌ | `.claude/agents/` | `.cursor/agents/` | ❌ | `.opencode/agents/` | built-in ❌ | `.github/copilot-instructions.md` | ❌ |
| MCPs | global ❌ | `.mcp.json` | `.cursor/mcp.json` | cursor global ❌ | `.opencode.json` | built-in ❌ | ❌ | ❌ |
| Commands | `.codex/skills/` | `.claude/skills/` | `.cursor/commands/` | ❌ | `.opencode/commands/` | `.agents/skills/` | ❌ | `<project>/.vscode/tasks.json` |

**Notes / Exceptions:**
* Codex tracking: LSP [#8745](https://github.com/openai/codex/issues/8745), Hooks [#2109](https://github.com/openai/codex/issues/2109), Subagents [#2604](https://github.com/openai/codex/issues/2604)
* Cursor Cloud Agents: uses Cursor global config only
* OpenCode MCPs: `~/.config/opencode/.opencode.json` → `mcpServers`
* AMP MCPs: `~/.config/amp/settings.json` → `amp.mcpServers` (OAuth in `~/.amp/oauth/`)
* Codex/Claude/AMP commands: stored under `skills/` for migrated installations
* VS Code LSP/config: extensions or settings
* Copilot: no user-defined MCPs/skills/commands
* VS Code user settings: macOS `~/Library/Application Support/Code/User/settings.json`, Linux `~/.config/Code/User/settings.json`
* Project root: `.vscode/`, `.github/`, `.codex/`, `.claude/`, `.cursor/`, `.opencode/`, `.agents/`
* Cursor rules vs skills: `.cursor/rules/` (apply modes), `.cursor/skills/` (agent-decided)
    

## Structure

```
configs/
├── agents/          # Agent definitions (verifier, askuserquestion)
├── codex-skills/    # Codex-specific skill imports kept as a separate install category
│   └── skills/
│       └── babysit-pr/
├── commands/        # Slash commands (commit, deslop, ultrathink, etc.)
├── cursor-plugins/  # Cursor plugin imports kept as a separate install category
│   ├── agents/
│   ├── hooks/
│   └── skills/
│       ├── cli-for-agents/
│       ├── continual-learning/
│       ├── create-learning-path/
│       ├── deslop/
│       ├── orchestrate/
│       ├── run-learning-retrospective/
│       ├── thermo-nuclear-code-quality-review/
│       └── what-did-i-get-done/
├── hooks/           # Optional generic hook packages
├── mcps/            # MCP server configurations
├── rules/           # Always-applied rules (clean-code, commit format, etc.)
├── skills/          # Modular skills for specific tasks
│   ├── bash-expert/
│   ├── changelog-generator/
│   ├── add-app-clip/
│   ├── audio-math-haptics/
│   ├── brandkit/
│   ├── building-native-ui/
│   ├── eas-update-insights/
│   ├── emil-design-eng/
│   ├── expo-api-routes/
│   ├── expo-*/
│   ├── find-skills/
│   ├── frontend-design/
│   ├── hallmark/
│   ├── diagnose/
│   ├── grill-with-docs/
│   ├── imagegen-frontend-web/
│   ├── taste-skill/
│   ├── tdd/
│   ├── mcp-builder/
│   ├── skill-creator/
│   └── ...
└── stacks/          # Language/framework-specific skill bundles
    ├── bun/
    ├── databases/
    ├── ios/         # SwiftUI, Swift concurrency, performance, Liquid Glass
    ├── nextjs/
    ├── react-native/
    ├── rust/
    ├── solana/
    ├── turborepo/
    └── typescript/
```

## Attributions

This repository contains configurations adapted from open-source projects. Below are the attributions for code copied or adapted from external sources.

| File | Description | Notes | Source | License |
|------|-------------|-------|--------|---------|
| `configs/skills/mcp-builder/` | MCP Server Development Guide - creating high-quality MCP servers |  | [anthropics/skills](https://github.com/anthropics/skills) | Apache-2.0 |
| `configs/skills/skill-creator/` | Skill Creator Guide - creating effective Claude skills |  | [anthropics/skills](https://github.com/anthropics/skills) | Apache-2.0 |
| `configs/skills/frontend-design/` | Frontend Design - distinctive, production-grade UI creation |  | [anthropics/skills](https://github.com/anthropics/skills) | Apache-2.0 |
| `configs/skills/karpathy-guidelines` | Behavioral guidelines to reduce common LLM coding mistakes. |  | [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills) | MIT |
| `configs/skills/emil-design-eng/` | Emil Kowalski design engineering philosophy for UI polish, component design, animation decisions | Copied from `skills/emil-design-eng` at `ecf66bb`; no license file found in source | [emilkowalski/skill](https://github.com/emilkowalski/skill) | NOASSERTION |
| `configs/skills/audio-math-haptics/` | First-principles audio-coupled haptic and kinetic UI feedback | Copied from `skill/audio-math-haptics` at `dc2ba99` | [heyAyushh/audio-math-haptics](https://github.com/heyAyushh/audio-math-haptics) | MIT |
| `configs/skills/hallmark/` | Anti-AI-slop design skill for greenfield pages, audits, redesigns, and design extraction | Copied package payload (`SKILL.md` + `references/`) at `9aba10e`; frontmatter adapted for stacc validator | [nutlope/hallmark](https://github.com/nutlope/hallmark) | MIT |
| `configs/skills/add-app-clip/`, `configs/skills/building-native-ui/`, `configs/skills/eas-update-insights/`, `configs/skills/expo-*/`, `configs/skills/native-data-fetching/`, `configs/skills/upgrading-expo/`, `configs/skills/use-dom/` | Official Expo skills for App Clips, native UI, EAS, deployment, SDK upgrades, modules, data fetching, and DOM components | Copied from `plugins/expo/skills` at `956a92b`; frontmatter adapted for stacc validator | [expo/skills](https://github.com/expo/skills/tree/main/plugins/expo/skills) | MIT |
| `configs/skills/brandkit/`, `configs/skills/brutalist-skill/`, `configs/skills/gpt-tasteskill/`, `configs/skills/image-to-code-skill/`, `configs/skills/imagegen-frontend-*/`, `configs/skills/minimalist-skill/`, `configs/skills/output-skill/`, `configs/skills/redesign-skill/`, `configs/skills/soft-skill/`, `configs/skills/stitch-skill/`, `configs/skills/taste-skill*/` | Anti-slop frontend, image-generation, brand-kit, redesign, and output-completion skills | Copied from `skills/` at `339afcb`; frontmatter adapted for stacc validator | [Leonxlnx/taste-skill](https://github.com/Leonxlnx/taste-skill) | MIT |
| `configs/cursor-plugins/skills/continual-learning/`, `configs/cursor-plugins/skills/cli-for-agents/`, `configs/cursor-plugins/skills/create-learning-path/`, `configs/cursor-plugins/skills/run-learning-retrospective/`, `configs/cursor-plugins/skills/orchestrate/`, `configs/cursor-plugins/skills/thermo-nuclear-code-quality-review/`, `configs/cursor-plugins/skills/what-did-i-get-done/`, `configs/cursor-plugins/skills/deslop/`, `configs/cursor-plugins/agents/agents-memory-updater.md`, `configs/cursor-plugins/hooks/continual-learning/` | Cursor plugin skills plus the continual-learning agent and hook package | Copied from selected `cursor/plugins` paths at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins) | MIT |
| `configs/codex-skills/skills/babysit-pr/` | Codex PR babysitter skill for monitoring GitHub PR review feedback, CI, and mergeability | Copied from `.codex/skills/babysit-pr` at `c4e53d1`; frontmatter adapted for stacc validator | [openai/codex](https://github.com/openai/codex/tree/main/.codex/skills/babysit-pr) | Apache-2.0 |
| `configs/skills/diagnose/` | Disciplined diagnosis loop for hard bugs and performance regressions | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/grill-with-docs/` | Grilling session that challenges plans against the existing domain model and docs | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/triage/` | Issue triage through a role/state workflow | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/improve-codebase-architecture/` | Find codebase architecture deepening opportunities | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/setup-matt-pocock-skills/` | Scaffold per-repo agent-skill configuration | Promoted plugin skill; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/tdd/` | Test-driven development with a red-green-refactor loop | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/to-issues/` | Break plans into independently-grabbable issues | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/to-prd/` | Turn conversation context into a PRD for the project issue tracker | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/zoom-out/` | Ask for a higher-level map of unfamiliar code | Promoted plugin skill; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/prototype/` | Build throwaway prototypes for logic or UI design questions | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/caveman/` | Ultra-compressed communication mode | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/grill-me/` | Interview the user until a plan or design is fully resolved | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/handoff/` | Compact the current conversation into a handoff document | Promoted plugin skill; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/skills/write-a-skill/` | Create new agent skills with proper structure and resources | Copied from promoted plugin manifest | [mattpocock/skills](https://github.com/mattpocock/skills) | MIT |
| `configs/stacks/ios/swift-concurrency-expert/` | Swift 6.2+ concurrency review and remediation |  | [Dimillian/Skills](https://github.com/Dimillian/Skills) | MIT |
| `configs/stacks/ios/swiftui-view-refactor/` | SwiftUI view refactoring patterns |  | [Dimillian/Skills](https://github.com/Dimillian/Skills) | MIT |
| `configs/stacks/ios/swiftui-performance-audit/` | SwiftUI performance auditing and optimization |  | [Dimillian/Skills](https://github.com/Dimillian/Skills) | MIT |
| `configs/stacks/ios/swiftui-ui-patterns/` | SwiftUI UI patterns and best practices |  | [Dimillian/Skills](https://github.com/Dimillian/Skills) | MIT |
| `configs/stacks/ios/swiftui-liquid-glass/` | iOS 26+ Liquid Glass API implementation |  | [Dimillian/Skills](https://github.com/Dimillian/Skills) | MIT |
| `configs/stacks/ios/ios-debugger-agent/` | XcodeBuildMCP-based iOS debugging |  | [Dimillian/Skills](https://github.com/Dimillian/Skills) | MIT |
| `configs/commands/deslop.md` | Remove AI-generated code slop | Also seen in [fatih/dotfiles](https://github.com/fatih/dotfiles) and [moeru-ai/airi](https://github.com/moeru-ai/airi) (MIT). | [triggerdotdev/trigger.dev](https://github.com/triggerdotdev/trigger.dev) | Apache-2.0 |
| `configs/agents/askuserquestion.md` | AskUserQuestion tool description | Adapted from Claude Code's built-in tool descriptions and agent prompts. | Claude Code / Anthropic System Prompts | NOASSERTION |
| `configs/commands/explore.md` | File search specialist agent prompt | Documented in [Piebald-AI/claude-code-system-prompts](https://github.com/Piebald-AI/claude-code-system-prompts) (MIT). | Claude Code / Anthropic System Prompts | NOASSERTION |
| `configs/stacks/bun/bun.mdc` | Bun.js best practices |  | [sanjeed5/awesome-cursor-rules-mdc](https://github.com/sanjeed5/awesome-cursor-rules-mdc) | CC0-1.0 |
| `configs/stacks/typescript/` | TypeScript conventions |  | [sanjeed5/awesome-cursor-rules-mdc](https://github.com/sanjeed5/awesome-cursor-rules-mdc) | CC0-1.0 |
| `configs/stacks/bun/postgresql.mdc` | PostgreSQL guidelines |  | [sanjeed5/awesome-cursor-rules-mdc](https://github.com/sanjeed5/awesome-cursor-rules-mdc) | CC0-1.0 |
| `configs/rules/clean-code.mdc` | Clean code guidelines |  | [PatrickJS/awesome-cursorrules](https://github.com/PatrickJS/awesome-cursorrules) | CC0-1.0 |
| `configs/stacks/solana/` | Solana Dev Skills |  | [Solana Foundation](https://github.com/solana-foundation/solana-dev-skill) | MIT |
| `configs/commands/rebase.md` | Rebase the current branch to resolve/maybe Merge Conflicts |  | [Raine Virta - blog](https://raine.dev/blog/resolve-conflicts-with-claude) | NOASSERTION |
| `configs/commands/clean-gone.md` | Cleans up all git branches marked as [gone] (branches that have been deleted on the remote but still exist locally), including removing associated worktrees. |  | [Raine Virta - blog](https://raine.dev/blog/resolve-conflicts-with-claude) | NOASSERTION |
| `configs/commands/review-pr.md` | Review Pull request from GitHub | Local stacc command | Original / stacc | MIT |
| `configs/commands/visualize.md` | Mermaid diagram generation |  | [anthropics/claude-code](https://github.com/anthropics/claude-code/blob/main/plugins/code-review/commands/code-review.md) | [LICENSE](https://github.com/anthropics/claude-code/blob/main/LICENSE.md) |
| `configs/commands/onboard-new-developer.md` | Developer onboarding checklist |  | [anthropics/claude-code](https://github.com/anthropics/claude-code/blob/main/plugins/code-review/commands/code-review.md) | [LICENSE](https://github.com/anthropics/claude-code/blob/main/LICENSE.md) |
| `configs/commands/refactor.md` | Code refactoring checklist (refactor-code.md) |  | [anthropics/claude-code](https://github.com/anthropics/claude-code/blob/main/plugins/code-review/commands/code-review.md) | [LICENSE](https://github.com/anthropics/claude-code/blob/main/LICENSE.md) |
| `configs/commands/commit.md` | Git commit workflow (commit-only.md) |  | [kinopeee/cursorrules](https://github.com/kinopeee/cursorrules) | MIT |
| `configs/commands/commit-push.md` | Commit and push workflow |  | [kinopeee/cursorrules](https://github.com/kinopeee/cursorrules) | MIT |
| `configs/commands/commit-push-pr.md` | Commit, push, and PR workflow |  | [kinopeee/cursorrules](https://github.com/kinopeee/cursorrules) | MIT |
| `configs/rules/commit-message-format.mdc` | Conventional Commits format |  | [kinopeee/cursorrules](https://github.com/kinopeee/cursorrules) | MIT |
| `configs/rules/pr-message-format.mdc` | PR message format |  | [kinopeee/cursorrules](https://github.com/kinopeee/cursorrules) | MIT |
| `configs/rules/prompt-injection-gaurd.mdc` | External context injection defense (prompt-injection-guard.mdc) |  | [kinopeee/cursorrules](https://github.com/kinopeee/cursorrules) | MIT |
| `configs/commands/review.md` | Security-focused code review |  | [anthropics/claude-code-security-review](https://github.com/anthropics/claude-code-security-review) | MIT |
| `configs/commands/council.md` | Spawn multiple agents to deeply explore a codebase area before acting |  | [@shaoruu](https://shaoruu.io/cursor/council) | NOASSERTION |
| `configs/commands/iterate-browser.md` | Autonomously iterate on UI changes using console.log and browser tools |  | [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills) | Apache-2.0 |
| `configs/skills/changelog-generator/` | Changelog generation from git commits | Also found in [davila7/claude-code-templates](https://github.com/davila7/claude-code-templates) (MIT) and [skillcreatorai/Ai-Agent-Skills](https://github.com/skillcreatorai/Ai-Agent-Skills) (MIT). | [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills) | Apache-2.0 |
| `configs/commands/ultrathink.md` | Deep reasoning mode protocol | Local stacc command | Original / stacc | MIT |
| `configs/commands/init.md` | AGENTS.md initialization | Local stacc command | Original / stacc | MIT |
| `configs/agents/verifier.md` | Work verification agent | Local stacc agent | Original / stacc | MIT |
| `configs/stacks/rust/ownership/`, `error-handling/`, `concurrency/`, `zero-cost-abstractions/`, `type-driven-design/`, `performance/`, `agent-friendly-cli/`, `anti-patterns/`, `coding-guidelines/` | Rust skill system — layered guidance for ownership, errors, concurrency, types, performance, and agent-friendly CLI design | Adapted from the layered skill system structure; `agent-friendly-cli/` adapts the local Cursor plugin CLI-for-agents guidance for Rust binaries | [actionbook/rust-skills](https://github.com/actionbook/rust-skills) | MIT |

## License

MIT. [LICENSE](LICENSE).

Individual components retain their original licenses:
- Anthropic skills: Apache-2.0 (see `LICENSE.txt` in skill directories)
- Matt Pocock skills: MIT (see `LICENSE.txt` in skill directories)
- Audio Math Haptics and Hallmark: MIT (see `LICENSE.txt` in skill directories)
- Expo skills: MIT (see `LICENSE.txt` in skill directories)
- Taste Skill skills: MIT (see `LICENSE.txt` in skill directories)
- Cursor plugin imports: MIT (see `LICENSE.txt` in cursor-plugin skill and hook directories)
- Codex skill imports: Apache-2.0 (see `LICENSE.txt` in codex-skill directories)
- ComposioHQ imports: Apache-2.0
- Dimillian/Skills: MIT
- actionbook/rust-skills: MIT
- Other components: See individual source repositories
