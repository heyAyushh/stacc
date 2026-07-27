# stacc
Curated configs for an AI coding workflow with muscles.
  
![Cursor](https://img.shields.io/badge/Cursor-black?style=flat&logo=cursor) ![Claude Code](https://img.shields.io/badge/Claude_Code-cc785c?style=flat&logo=anthropic) ![Codex](https://img.shields.io/badge/Codex-10a37f?style=flat&logo=openai&logoColor=white) ![OpenCode](https://img.shields.io/badge/OpenCode-1a1a2e?style=flat&logo=go&logoColor=00ADD8) ![AMP](https://img.shields.io/badge/AMP-ff5543?style=flat&logo=sourcegraph&logoColor=white)

![stacc banner](stacc.png)

## Installation

### Quick Install

```bash
curl -fsSL https://stacc.fyi/install.sh | bash
```

**Or using GitHub URL:**
```bash
curl -fsSL https://stacc.fyi/install.sh | bash
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
- **Category selection**: commands, rules, agents, skills, stack, hooks, mcps, cursor-plugins, codex-skills, codex-plugins
- **Stack selection**: choose one or more stack skill folders from `configs/stacks/`
- **Hook and MCP selection**: choose hook packages and MCP servers from the Hooks/MCP segment
- **Managed updates**: update or uninstall skills, stacks, and Codex plugins previously installed by stacc
- **Version actions**: refresh git status, sync skill metadata, run checks, or bootstrap/upgrade the binary

#### Stacks

Stacks are focused domain, workflow, framework, or language bundles under
`configs/stacks/`. When you select the `stack` category, the installer prompts
you to choose one or more stack folders and installs them into each editor's
`skills/` directory. Each stack `SKILL.md` is a router; child skills are loaded
only when their branch matches the task. CLI installs require an explicit
`--stack <name>` or `--stack all`; selecting a stack in the TUI enables the
category automatically.

#### Installer options

`install.sh` is now a bootstrap and legacy-flag adapter. It prefers a prebuilt GitHub release binary for the current platform, then falls back to Cargo only when no matching binary is available. These commands install or run `stacc`, then forward to the Rust install engine:

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

```bash
./install.sh install --editor codex --codex-plugin lazycodex --dry-run --print-plan
```

```bash
./install.sh sync --editor codex --scope project --dry-run --print-plan
```

```bash
./install.sh update --editor codex --skill ultragoal --dry-run --print-plan
```

```bash
./install.sh uninstall --editor codex --codex-plugin lazycodex --dry-run --print-plan
```

### Rust Control Panel

The Rust TUI gives stacc a thin control panel for installation, customization, version metadata, hooks, and MCP selection.

Install the binary directly:

```bash
cargo install --git https://github.com/heyAyushh/stacc --locked --force
```

`install.sh` expects release archives named `stacc-<target>.tar.gz`, containing `stacc` or `stacc.exe`. Supported targets are:

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

Override the release source with `STACC_RELEASE_REPO=owner/repo` or pin a version with `STACC_RELEASE_TAG=vX.Y.Z`.

For a local checkout:

```bash
cargo install --path . --locked --force
```

The installed binary bundles `install.sh`, `configs/`, and README metadata, so agents do not need to pass `--root` after `cargo install`. It materializes the bundle into a versioned cache path on first run. Set `STACC_BUNDLE_ROOT=/path/to/cache` when an agent needs a deterministic bundle directory.

Human default:

```bash
stacc
```

Local checkout equivalent:

```bash
cargo run
cargo run -- --panel
```

Agent/non-interactive commands:

```bash
stacc status --json
stacc install --editor cursor --editor codex --scope project --category rules --category skills --dry-run --print-plan
stacc install --editor cursor --scope project --category hooks --hook continual-learning --dry-run --print-plan
stacc install --editor codex --scope global --category rules --category skills --category mcps --mcp-server github --yes
stacc install --editor codex --codex-plugin lazycodex --dry-run --print-plan
stacc sync --editor codex --scope project --dry-run --print-plan
stacc sync --editor codex --scope project --skill ultragoal --dry-run --print-plan
stacc sync --editor codex --codex-plugin lazycodex --dry-run --print-plan
stacc update --editor codex --skill ultragoal --dry-run --print-plan
stacc update --editor codex --codex-plugin lazycodex --dry-run --print-plan
stacc uninstall --editor codex --skill ultragoal --dry-run --print-plan
stacc uninstall --editor codex --codex-plugin lazycodex --dry-run --print-plan
stacc sync-metadata --refresh-origin
stacc sync-metadata --refresh-origin --dry-run --json
stacc sync-metadata --refresh-origin --dry-run --fail-on-outdated
stacc bootstrap --dry-run
stacc check
```

Local checkout form prefixes the same commands with `cargo run --`:

```bash
cargo run -- install --editor cursor --scope project --category hooks --hook continual-learning --dry-run --print-plan
```

#### Apple container environment

On an Apple silicon Mac running macOS 26 or newer, the repository can build and
test inside [Apple container](https://github.com/apple/container). The tracked
image includes pinned Rust, Node.js, and ShellCheck toolchains plus Git, Clippy,
and rustfmt.

Start the Apple container service once, then use the repository wrapper:

```bash
container system start
./container.sh build
./container.sh check
./container.sh docs
```

`./container.sh shell` opens an interactive shell in the same image.
`STACC_CONTAINER_IMAGE`, `STACC_CONTAINER_CPUS`, and
`STACC_CONTAINER_MEMORY` override the image name and default resource limits.
Each run mounts the checkout read-only and works from an ephemeral copy, so
Linux build outputs do not overwrite the host checkout.
The image is OCI-compatible; `container.sh` intentionally uses Apple
`container build` and `container run` rather than requiring Docker Desktop.

#### TUI parity

| Feature | TUI | CLI |
|---------|-----|-----|
| Editor/scope/conflict/dry-run selection | Install segment | `stacc install --editor ... --scope ... --conflict ... --dry-run` |
| Category and stack selection | Customise segment | `--category ... --stack ...` |
| Hook package selection | Hooks/MCP segment | `--category hooks --hook ...` |
| MCP server selection | Hooks/MCP segment | `--category mcps --mcp-server ...` |
| Codex plugin selection | Hooks/MCP segment | `--codex-plugin ...` |
| Managed skill/plugin ownership | CLI only | `stacc sync ...`, `stacc update ...`, `stacc uninstall ...` |
| Git/status metadata | Version and Skills segments | `stacc status`, `stacc sync-metadata` |
| Checks | Version segment | `stacc check` |
| Self install/upgrade | Version segment | `stacc bootstrap` |

Panel selectors keep their categories in sync. Enabling Hooks or MCPs selects
the currently available entries, while clearing the last entry disables that
category. The panel dry-run switch also applies to metadata sync and bootstrap.
Invalid install selections stay in the panel with an actionable message.

Panel actions return to the TUI with a result message after install dry-runs,
installs, metadata audits or syncs, checks, and bootstrap dry-runs.

#### Conflict modes

| Mode | Behavior |
|------|----------|
| `backup` | Move conflicting targets to `.bak.<timestamp>` before writing |
| `overwrite` | Replace conflicting targets |
| `skip` | Leave conflicting targets unchanged |
| `selective` | Show prompt operations in dry-run plans and prompt per conflicting file in an interactive terminal |

Use `backup`, `overwrite`, `skip`, or `--dry-run` for non-interactive agents.

#### Metadata and defaults

- Install execution is native Rust: file copying, conflict handling, rules summaries, hook package filtering, MCP JSON/TOML merge, and installed-binary smoke checks use typed Rust planning with explicit dry-run/yes gates.
- Installs write a stacc ownership manifest at `<target-root>/.stacc/manifest.json`. `stacc update` and `stacc uninstall` only operate on skill, stack, and Codex plugin entries recorded there by stacc; they do not infer ownership from arbitrary files already present in an editor directory. A skipped conflict or selectively preserved package is not newly claimed as managed; backup and overwrite resolve managed packages at the package-directory boundary before ownership is recorded.
- Use `stacc sync --editor ... --scope ... --dry-run --print-plan` to backfill the ownership manifest for stacc skill folders that are already installed. With no `--skill`, it scans the small core in `configs/skills`, stack folders, editor adapters under `configs/plugins`, and command skills for editors that store commands as skills. Codex plugin backfill is explicit with `--codex-plugin` because Codex owns the plugin installation state.
- Optional Codex plugins live in `configs/plugins/codex/plugins.json`. Explicit `--codex-plugin` keys imply the internal `codex-plugins` category and a global Codex target, so `--scope global` is not required. The installer plans fixed `codex plugin marketplace add ...` and `codex plugin add ...` commands, then runs them only with `--yes`. Managed updates use `codex plugin marketplace upgrade ...` plus `codex plugin add ...`; managed uninstalls use `codex plugin remove ...` and remove the marketplace when no other stacc-managed plugin entry uses it.
- Metadata sync writes `configs/metadata/skills.lock.json` with each skill's local path, license, version, source URL, declared origin commit, and current upstream repo HEAD commit when GitHub lookup is enabled. Its report includes `outdated`, `outdated_count`, and an `outdated_sources` table with `freshness_scope: "repo-head"`, computed by comparing the declared imported commit with the current upstream repository HEAD. Use `stacc sync-metadata --refresh-origin --dry-run --json` for a non-mutating freshness report, or add `--fail-on-outdated` when CI should block on stale pinned source snapshots.
- Source freshness is a maintainer signal, not a payload updater. When `outdated` is true, review the upstream diff, refresh the vendored files under `configs/`, update attribution/license metadata, then run `stacc check`.
- Custom panel defaults live in `configs/stacc-panel.json`.
- `stacc bootstrap` matches the shell bootstrap path by running `cargo install --git https://github.com/heyAyushh/stacc.git --locked --force`; `--dry-run` prints the command without network access.

#### Rust module layout

| Module | Responsibility |
|--------|----------------|
| `src/main.rs` | CLI parsing and command dispatch |
| `src/panel.rs` | Ratatui control panel state and screens |
| `src/install.rs` | Typed install planning and operation execution |
| `src/hook_selection.rs` | Hook package discovery and filtering |
| `src/selective.rs` | Interactive selective conflict prompts |
| `src/bootstrap.rs` | `cargo install --git` bootstrap command |
| `src/metadata.rs` | Skill license/version/origin lockfile sync |
| `src/check.rs` | Format, test, lint, shell, JSON, install, and smoke checks |
| `src/bundle.rs` | Embedded runtime bundle materialization |
| `src/catalog.rs` | Config catalog discovery and install compatibility |

### Checks

Run the full local gate before pushing installer or control-panel changes:

```bash
stacc check
```

The gate runs Rust format checks, tests, clippy, installer syntax checks, MCP/panel/skill metadata JSON validation, `cargo install --path`, and smoke checks against the installed `stacc` binary. The smoke checks include metadata report generation, managed skill install, manifest backfill, update dry-run, and uninstall dry-run in `target/stacc-check`. It also runs `shellcheck -x install.sh` when `shellcheck` is installed. Use `stacc check --require-shellcheck` when CI must fail if `shellcheck` is missing.

The `ci` workflow runs `stacc check` on pushes and pull requests. It also runs a scheduled source freshness report with `stacc sync-metadata --refresh-origin --dry-run --json`; that job reports whether vendored external sources are outdated without auto-updating them.

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
├── commands/        # Slash commands (commit, deslop, ultrathink, etc.)
├── hooks/           # Optional generic hook packages
├── mcps/            # MCP server configurations
├── plugins/         # Editor-specific adapters only
│   ├── codex/       # babysit-pr skill + optional marketplace catalog
│   └── cursor/      # continual-learning + orchestrate skills, agents, hooks
├── rules/           # Always-applied rules (clean-code, commit format, etc.)
├── skills/          # Small, frequently used cross-domain core
│   ├── agent-browser/
│   ├── diagnose/
│   ├── find-skills/
│   ├── gui-automation/
│   ├── handoff/
│   ├── skill-creator/
│   ├── tdd/
│   ├── ultragoal/
│   ├── using-git-worktrees/
│   └── writing-great-skills/
└── stacks/          # Focused domain/framework/language bundles
    ├── bun/
    ├── databases/
    ├── design/
    ├── engineering/
    ├── expo/
    ├── ios/         # SwiftUI, Swift concurrency, performance, Liquid Glass
    ├── nextjs/
    ├── productivity/
    ├── react-native/
    ├── review/
    ├── rust/
    ├── solana/
    ├── turborepo/
    └── typescript/
```

## Attributions

This repository contains configurations adapted from open-source projects. Below are the attributions for code copied or adapted from external sources.
The package-by-package inventory, including stacc-authored packages and non-canonical tracked mirrors, is maintained in [`configs/metadata/source-attribution-audit.md`](configs/metadata/source-attribution-audit.md).

| File | Description | Notes | Source | License |
|------|-------------|-------|--------|---------|
| `configs/stacks/nextjs/react-best-practices/` | React and Next.js performance guidance from Vercel Engineering | Copied from `skills/react-best-practices` at `7c180d9` | [vercel-labs/agent-skills](https://github.com/vercel-labs/agent-skills/tree/main/skills/react-best-practices) | MIT |
| `configs/stacks/nextjs/web-interface-guidelines/` | Web interface, accessibility, and UI review guidance | Copied from the Web Interface Guidelines package at `4e799d4` | [vercel-labs/web-interface-guidelines](https://github.com/vercel-labs/web-interface-guidelines) | MIT |
| `configs/commands/skills/*` | Stacc command-as-skill packages generated from the tracked command library | Local stacc-authored wrappers; installable as skills for editors that use skill folders | local stacc | MIT |
| `configs/skills/agent-browser/`, `configs/stacks/engineering/bash-expert/`, `configs/skills/find-skills/`, `configs/skills/using-git-worktrees/` | Stacc workflow skills | Local stacc-authored packages | local stacc | MIT |
| `configs/stacks/bun/`, `configs/stacks/databases/`, `configs/stacks/design/`, `configs/stacks/engineering/`, `configs/stacks/expo/`, `configs/stacks/ios/`, `configs/stacks/nextjs/`, `configs/stacks/productivity/`, `configs/stacks/react-native/`, `configs/stacks/review/`, `configs/stacks/turborepo/` | Stacc stack routers and locally authored stack skills | Local routers are MIT; imported child packages and CC0 rule content are attributed separately below | local stacc | MIT; imported components retain the licenses listed below |
| `configs/stacks/engineering/mcp-builder/` | MCP Server Development Guide - creating high-quality MCP servers |  | [anthropics/skills](https://github.com/anthropics/skills) | Apache-2.0 |
| `configs/skills/skill-creator/` | Skill Creator Guide - creating effective Claude skills |  | [anthropics/skills](https://github.com/anthropics/skills) | Apache-2.0 |
| `configs/stacks/design/frontend-design/` | Frontend Design - distinctive, production-grade UI creation |  | [anthropics/skills](https://github.com/anthropics/skills) | Apache-2.0 |
| `configs/stacks/review/karpathy-guidelines/` | Behavioral guidelines to reduce common LLM coding mistakes. |  | [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills) | MIT |
| `configs/stacks/design/emil-design-eng/` | Emil Kowalski design engineering philosophy for UI polish, component design, animation decisions | Copied from `skills/emil-design-eng` at `ecf66bb`; upstream repository later moved from `emilkowalski/skill` to `emilkowalski/skills` and publishes an MIT license | [emilkowalski/skills](https://github.com/emilkowalski/skills/tree/main/skills/emil-design-eng) | MIT |
| `configs/stacks/design/apple-design/` | Apple interface-design and fluid-motion principles translated for web implementation | Copied from `skills/apple-design` at `56de6f5`; provenance frontmatter added for stacc metadata | [emilkowalski/skills](https://github.com/emilkowalski/skills/tree/main/skills/apple-design) | MIT |
| `.agents/skills/review-animations/` | Strict animation and motion review guidance | Project-local adapted mirror of Emil Kowalski's review skill; not part of the canonical installer catalog | [emilkowalski/skills](https://github.com/emilkowalski/skills/tree/main/skills/review-animations) | MIT |
| `configs/stacks/ios/audio-math-haptics/` | First-principles audio-coupled haptic and kinetic UI feedback | Copied from `skill/audio-math-haptics` at `dc2ba99` | [heyAyushh/audio-math-haptics](https://github.com/heyAyushh/audio-math-haptics) | MIT |
| `configs/stacks/design/hallmark/` | Anti-AI-slop design skill for greenfield pages, audits, redesigns, and design extraction | Copied package payload (`SKILL.md` + `references/`) at `9aba10e`; frontmatter adapted for stacc validator | [nutlope/hallmark](https://github.com/nutlope/hallmark) | MIT |
| `configs/stacks/review/react-doctor/` | React diagnostics skill for scanner-backed cleanup, triage, and rule explanation workflows | Copied from `skills/react-doctor` at `0b64af58`; frontmatter adapted for stacc validator | [millionco/react-doctor](https://github.com/millionco/react-doctor/tree/main/skills/react-doctor) | LicenseRef-Million-Modified-MIT |
| `configs/stacks/ios/add-app-clip/`, `configs/stacks/expo/*/` | Official Expo skills for App Clips, native UI, EAS, deployment, SDK upgrades, modules, data fetching, and DOM components | Copied from `plugins/expo/skills` at `956a92b`; frontmatter adapted for stacc validator | [expo/skills](https://github.com/expo/skills/tree/main/plugins/expo/skills) | MIT |
| `configs/stacks/engineering/cli-for-agents/references/agent-browser-runtime-skills.md` | Reference pattern for versioned, CLI-served agent instructions | Summarizes the current agent-browser discovery-skill/runtime-skill architecture | [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser) | Apache-2.0 |
| `configs/skills/ultragoal/` | Durable Codex goal design and activation workflow | Copied from `agents/skills/ultragoal` at `1eb180f`; upstream publishes no repository license, so no reuse license is asserted here | [jxnl/dots](https://github.com/jxnl/dots/tree/master/agents/skills/ultragoal) | LicenseRef-No-Published-License |
| `configs/stacks/design/brandkit/`, `configs/stacks/design/brutalist-skill/`, `configs/stacks/design/gpt-tasteskill/`, `configs/stacks/design/image-to-code-skill/`, `configs/stacks/design/imagegen-frontend-*/`, `configs/stacks/design/minimalist-skill/`, `configs/stacks/productivity/output-skill/`, `configs/stacks/design/redesign-skill/`, `configs/stacks/design/soft-skill/`, `configs/stacks/design/stitch-skill/`, `configs/stacks/design/taste-skill*/` | Anti-slop frontend, image-generation, brand-kit, redesign, and output-completion skills | Copied from `skills/` at `339afcb`; frontmatter adapted for stacc validator | [Leonxlnx/taste-skill](https://github.com/Leonxlnx/taste-skill) | MIT |
| `configs/stacks/engineering/cli-for-agents/` | Agent-friendly CLI design guidance | Copied from `cli-for-agent/skills/cli-for-agents` at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins/tree/main/cli-for-agent/skills/cli-for-agents) | MIT |
| `configs/plugins/cursor/skills/continual-learning/` | Continual learning skill for transcript-derived memory updates | Copied from `continual-learning/skills/continual-learning` at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins/tree/main/continual-learning/skills/continual-learning) | MIT |
| `configs/stacks/productivity/create-learning-path/` | Create a learning path from current work context | Copied from `teaching/skills/create-learning-path` at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins/tree/main/teaching/skills/create-learning-path) | MIT |
| `configs/stacks/productivity/run-learning-retrospective/` | Run a learning retrospective from completed work | Copied from `teaching/skills/run-learning-retrospective` at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins/tree/main/teaching/skills/run-learning-retrospective) | MIT |
| `configs/plugins/cursor/skills/orchestrate/` | Multi-agent orchestration workflow | Copied from `orchestrate/skills/orchestrate` at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins/tree/main/orchestrate/skills/orchestrate) | MIT |
| `configs/stacks/review/thermo-nuclear-code-quality-review/` | Strict code-quality review skill | Copied from `cursor-team-kit/skills/thermo-nuclear-code-quality-review` at `21327be`; identical payload also exists in `thermos/skills/thermo-nuclear-code-quality-review` | [cursor/plugins](https://github.com/cursor/plugins/tree/main/cursor-team-kit/skills/thermo-nuclear-code-quality-review) | MIT |
| `configs/stacks/productivity/what-did-i-get-done/` | Work-summary skill | Copied from `cursor-team-kit/skills/what-did-i-get-done` at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins/tree/main/cursor-team-kit/skills/what-did-i-get-done) | MIT |
| `configs/stacks/review/deslop/` | Deslop skill from Cursor Team Kit | Copied from `cursor-team-kit/skills/deslop` at `21327be`; frontmatter adapted for stacc validator | [cursor/plugins](https://github.com/cursor/plugins/tree/main/cursor-team-kit/skills/deslop) | MIT |
| `configs/plugins/cursor/agents/agents-memory-updater.md`, `configs/plugins/cursor/hooks/continual-learning/` | Continual-learning agent and hook package | Copied from `continual-learning/agents` and `continual-learning/hooks` at `21327be` | [cursor/plugins](https://github.com/cursor/plugins/tree/main/continual-learning) | MIT |
| `configs/plugins/codex/skills/babysit-pr/` | Codex PR babysitter skill for monitoring GitHub PR review feedback, CI, and mergeability | Copied from `.codex/skills/babysit-pr` at `c4e53d1`; frontmatter adapted for stacc validator | [openai/codex](https://github.com/openai/codex/tree/main/.codex/skills/babysit-pr) | Apache-2.0 |
| `configs/plugins/codex/plugins.json` | Optional LazyCodex Codex plugin marketplace entry | References `code-yeongyu/lazycodex` as an opt-in Codex marketplace source; no LazyCodex payload is vendored | [code-yeongyu/lazycodex](https://github.com/code-yeongyu/lazycodex) | MIT |
| `configs/skills/gui-automation/` | GUI automation workflow for visual interaction, screenshots, and end-to-end QA with CUA | Copied from `skills/gui-automation` at `73fe822`; command examples retained as documentation only | [trycua/cua](https://github.com/trycua/cua/tree/main/skills/gui-automation) | MIT |
| `configs/stacks/review/ponytail/`, `configs/stacks/review/ponytail-*` | Minimalist coding mode plus over-engineering review, audit, debt, gain, and help skills | Copied from `skills/` at `40e50d9`; frontmatter adapted for stacc validator | [DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail) | MIT |
| `configs/skills/diagnose/` | Disciplined diagnosis loop for hard bugs and performance regressions | Copied from `skills/engineering/diagnosing-bugs`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/diagnosing-bugs) | MIT |
| `configs/stacks/engineering/grill-with-docs/` | Grilling session that challenges plans against the existing domain model and docs | Copied from `skills/engineering/grill-with-docs`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/grill-with-docs) | MIT |
| `configs/stacks/engineering/triage/` | Issue triage through a role/state workflow | Copied from `skills/engineering/triage`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/triage) | MIT |
| `configs/stacks/engineering/improve-codebase-architecture/` | Find codebase architecture deepening opportunities | Copied from `skills/engineering/improve-codebase-architecture`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/improve-codebase-architecture) | MIT |
| `configs/skills/tdd/` | Test-driven development with a red-green-refactor loop | Copied from `skills/engineering/tdd`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/tdd) | MIT |
| `configs/stacks/engineering/to-issues/` | Break plans into independently-grabbable issues | Copied from `skills/engineering/to-issues`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/to-issues) | MIT |
| `configs/stacks/engineering/to-prd/` | Turn conversation context into a PRD for the project issue tracker | Copied from `skills/engineering/to-prd`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/to-prd) | MIT |
| `configs/stacks/productivity/zoom-out/` | Ask for a higher-level map of unfamiliar code | Adapted from the root `zoom-out` package on the historical `v1` branch at `8a54bc3` | [mattpocock/skills v1](https://github.com/mattpocock/skills/tree/v1/zoom-out) | MIT |
| `configs/stacks/engineering/prototype/` | Build throwaway prototypes for logic or UI design questions | Copied from `skills/engineering/prototype`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/prototype) | MIT |
| `configs/stacks/productivity/caveman/` | Ultra-compressed communication mode | Exact copy of the root `caveman` package on the historical `v1` branch at `8a54bc3` | [mattpocock/skills v1](https://github.com/mattpocock/skills/tree/v1/caveman) | MIT |
| `configs/stacks/productivity/grill-me/` | Interview the user until a plan or design is fully resolved | Exact copy of the root `grill-me` package on the historical `v1` branch at `8a54bc3` | [mattpocock/skills v1](https://github.com/mattpocock/skills/tree/v1/grill-me) | MIT |
| `configs/skills/handoff/` | Compact the current conversation into a handoff document | Copied from `skills/productivity/handoff`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/productivity/handoff) | MIT |
| `configs/stacks/engineering/write-a-skill/` | Create new agent skills with proper structure and resources | Exact copy of the root `write-a-skill` package on the historical `v1` branch at `8a54bc3` | [mattpocock/skills v1](https://github.com/mattpocock/skills/tree/v1/write-a-skill) | MIT |
| `.agents/skills/setup-matt-pocock-skills/`, `.claude/skills/setup-matt-pocock-skills/`, `.codex/skills/setup-matt-pocock-skills/`, `.cursor/skills*/setup-matt-pocock-skills/`, `.opencode/skills*/setup-matt-pocock-skills/` | Tool-specific installed mirrors of the Matt Pocock setup skill | Tracked legacy mirrors; canonical installer source is not present under `configs/` | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/engineering/setup-matt-pocock-skills) | MIT |
| `configs/skills/writing-great-skills/` | Reference vocabulary and principles for writing predictable skills | Copied from `skills/productivity/writing-great-skills` at `d574778`; frontmatter adapted for stacc validator | [mattpocock/skills](https://github.com/mattpocock/skills/blob/main/skills/productivity/writing-great-skills/SKILL.md) | MIT |
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
| `configs/stacks/engineering/changelog-generator/` | Changelog generation from git commits | Also found in [davila7/claude-code-templates](https://github.com/davila7/claude-code-templates) (MIT) and [skillcreatorai/Ai-Agent-Skills](https://github.com/skillcreatorai/Ai-Agent-Skills) (MIT). | [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills/tree/master/changelog-generator) | Apache-2.0 |
| `configs/commands/ultrathink.md` | Deep reasoning mode protocol | Local stacc command | Original / stacc | MIT |
| `configs/commands/init.md` | AGENTS.md initialization | Local stacc command | Original / stacc | MIT |
| `configs/agents/verifier.md` | Work verification agent | Local stacc agent | Original / stacc | MIT |
| `configs/stacks/rust/` | Rust skill system — layered guidance for ownership, errors, concurrency, types, performance, and agent-friendly CLI design | Adapted from the upstream `skills/` subtree; local folder names were normalized, and `agent-friendly-cli/` also adapts the local Cursor plugin CLI-for-agents guidance for Rust binaries | [actionbook/rust-skills](https://github.com/actionbook/rust-skills/tree/main/skills) | MIT |

## License

MIT. [LICENSE](LICENSE).

Individual components retain their original licenses:
- Anthropic skills: Apache-2.0 (see `LICENSE.txt` in skill directories)
- Matt Pocock skills: MIT (see `LICENSE.txt` in skill directories)
- Audio Math Haptics and Hallmark: MIT (see `LICENSE.txt` in skill directories)
- Emil Kowalski skills: MIT (see `LICENSE.txt` in skill directories)
- Expo skills: MIT (see `LICENSE.txt` in skill directories)
- Taste Skill skills: MIT (see `LICENSE.txt` in skill directories)
- CUA and Ponytail skills: MIT (see `LICENSE.txt` in skill directories)
- Cursor plugin imports: MIT (see `LICENSE.txt` in cursor-plugin skill and hook directories)
- Codex skill imports: Apache-2.0 (see `LICENSE.txt` in codex-skill directories)
- LazyCodex optional marketplace reference: MIT
- ComposioHQ imports: Apache-2.0
- Dimillian/Skills: MIT
- actionbook/rust-skills: MIT
- Vercel Labs skill imports: MIT
- Other components: See individual source repositories
