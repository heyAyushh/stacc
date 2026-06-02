---
name: rust-agent-friendly-cli
description: Designs Rust command-line interfaces that are reliable for agents, scripts, and humans. Use when building or reviewing Rust binaries, clap command trees, diagnostics commands, codegen tools, or automation-facing workflows.
---

# Agent-Friendly CLI

> **Layer 2: Interface Design**

## Core Question

**Can another agent run this command correctly without an interactive back-and-forth?**

Before adding or changing a CLI:
- Are all required decisions available as flags or stdin?
- Does `--help` show the exact working examples?
- Do failures tell the caller how to retry?

---

## Thinking Prompt

1. **What is the automation contract?**
   - Inputs: path, stdin, config file, environment, or all of them
   - Outputs: human text, JSON, files, or exit status
   - Side effects: read-only, write, network, or destructive

2. **Can it run unattended?**
   - No prompts on the default path
   - Every prompt has an explicit flag alternative
   - Destructive commands have `--dry-run` plus `--yes` or `--force`

3. **Can failures be fixed from the message?**
   - Explain what was wrong
   - Name the expected input shape
   - Include one or two exact retry commands

---

## CLI Contract Checklist

| Area | Requirement |
|------|-------------|
| Input | Accept paths and stdin (`--stdin` or `--input -`) when useful |
| Identity | Preserve virtual source names with flags like `--stdin-path` |
| Output | Offer structured output such as `--json` for automation |
| Help | Put real examples in top-level and subcommand help |
| Errors | Return usage errors before doing work |
| Safety | Require `--dry-run` and explicit confirmation for writes |
| Exit codes | Keep stable meanings and document them |

## Rust Implementation Patterns

| Need | Rust Pattern |
|------|--------------|
| Command parsing | `clap::Parser` or `clap::CommandFactory` |
| Example-rich help | `after_help` / `after_long_help` constants |
| Usage validation | Dedicated validation function before execution |
| Stdin support | Read `std::io::stdin()` once at the command boundary |
| Structured output | Serialize response types with `serde` |
| Actionable errors | Small error type with `Display` examples |
| Testing | `assert_cmd`, snapshot tests, or parser unit tests |

## Exit Code Guide

| Code | Meaning |
|------|---------|
| 0 | Command succeeded |
| 1 | Command ran and found reportable issues or runtime failure |
| 2 | Caller used the command incorrectly |

Document any project-specific differences near the CLI help and tests.

---

## Example Help Shape

```text
Examples:
  stacc
  stacc status --json
  stacc install --editor codex --scope global --category rules --category skills --dry-run
  stacc install --editor codex --scope global --category rules --category skills --yes
  stacc sync-metadata --refresh-origin
  stacc check

Output:
  Exit 0 when the command succeeds.
  Exit 1 when the command cannot complete.
  Exit 2 for invalid command usage.
```

## Error Message Shape

```text
Error: install requires at least one --editor and one --category.

Examples:
  stacc install --editor codex --scope global --category rules --category skills --dry-run
  stacc install --editor codex --scope global --category rules --category skills --yes
```

## Installability Contract

Cargo-installed binaries must work outside the checkout. For stacc, the binary bundles its installer and configs, then materializes them at runtime.

```bash
cargo install --git https://github.com/heyAyushh/stacc --locked --force
stacc
stacc status --json
```

Use `STACC_BUNDLE_ROOT=/path/to/cache` when an agent needs deterministic bundle output.

## Project Checks

For this repo, use the single check entrypoint:

```bash
stacc check
```

It runs:
- `cargo fmt --all -- --check`
- `cargo test`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `bash -n install.sh`
- `shellcheck -x install.sh` when `shellcheck` is installed
- Rust JSON validation for MCP, panel, and skill metadata files
- `cargo install --path` plus installed-binary smoke checks

Use this stricter CI shape when shell lint is required:

```bash
stacc check --require-shellcheck
```

## Quick Review Checklist

- [ ] No required interactive prompts on automation paths
- [ ] Every required input can be passed as a flag or stdin
- [ ] `--help` includes real project commands, not placeholders
- [ ] Usage validation happens before expensive work
- [ ] Errors include expected shape plus retry examples
- [ ] JSON or another stable machine format exists when output is consumed
- [ ] Destructive paths support `--dry-run` and explicit confirmation
- [ ] Help and usage errors are covered by tests

---

## Common Mistakes

| Mistake | Why Wrong | Better |
|---------|-----------|--------|
| Prompting for required input | Agents cannot reliably answer prompts | Require flags or stdin |
| Hiding examples in README only | Callers see `--help` first | Put examples in CLI help |
| Free-form output only | Scripts need stable fields | Add `--json` or a schema |
| Generic errors | Caller cannot self-correct | Include exact retry command |
| Late validation | Wastes work before failing | Validate command shape first |
| Destructive default | Easy to damage worktrees | Default to dry-run or require consent |

---

## Related Skills

- `error-handling` - model usage errors and runtime failures cleanly
- `coding-guidelines` - keep command modules idiomatic
- `anti-patterns` - catch unwraps, panics, and hidden side effects
