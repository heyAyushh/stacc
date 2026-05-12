# Changelog

All notable changes to stacc are documented here.

---

## [Unreleased]

### Added
- **Rust skill system** — 8 layered skill subdirectories inside `configs/stacks/rust/`, adapted from [actionbook/rust-skills](https://github.com/actionbook/rust-skills):
  - `ownership/` — borrow/move error guidance, lifetime design
  - `error-handling/` — Result vs Option vs panic, anyhow vs thiserror
  - `concurrency/` — Send/Sync, threads vs async, shared state patterns
  - `zero-cost-abstractions/` — generics vs dyn Trait, dispatch tradeoffs
  - `type-driven-design/` — newtype, type state, PhantomData, builder
  - `performance/` — profiling workflow, allocation reduction, benchmarking
  - `anti-patterns/` — code review checklist, deprecated crate table
  - `coding-guidelines/` — 50 core style rules for naming, strings, memory, async

---

## Recent Highlights

### Skills & Agents
- **agent-browser skill** — browser automation and session management
- **git-engineer agent** — orchestrates complex git workflows
- **commit skill** — intelligent change analysis for structured commits
- **karpathy-guidelines skill** — behavioral rules to reduce common LLM coding mistakes

### Stacks
- **iOS stack** — SwiftUI patterns, Swift concurrency, performance audit, Liquid Glass
- **Rust stack** — idiomatic Rust best practices (structure, patterns, error handling)
- **Solana stack** — programs, frontend kit, IDL codegen, security, testing
- **Turborepo stack** — monorepo caching, filtering, CI patterns
- **TypeScript stack** — conventions, advanced patterns, Biome linting

### Installer Improvements
- Conflict resolution with overwrite / backup / skip / selective modes
- MCP config merging via `jq` deep-merge when target exists
- Target path shown in conflict resolution prompts
- Fixed shared root conflict handling for multi-tool installs
- Support for AMP Code, OpenCode, and Codex tool targets

### Commands & Skills
- Migrated commands to skills architecture
- Added `council`, `iterate-browser`, `using-git-worktrees`, `changelog-generator`, `fix-pr`, `commit-push`, `commit-push-pr`, `deslop`, `simplify`, `refactor`, `visualize`, `explore`, `rebase`, `clean-gone`
- `ultrathink` deep reasoning mode
- `init` for AGENTS.md initialization
- `frontend-design` for production-grade UI generation
- `mcp-builder` for building MCP servers
- `skill-creator` for creating new skills

---

## Format

This changelog follows [Keep a Changelog](https://keepachangelog.com/) conventions.
