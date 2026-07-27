# Rust control plane

## Scope

This directory owns the `stacc` CLI, TUI, install planning, execution, metadata
sync, and repository checks. Keep product semantics here; `install.sh` is only
the bootstrap and legacy-flag adapter.

## Module map

- `main.rs`: Clap arguments and command dispatch. Build requests here, but keep
  install behavior in `install.rs`.
- `install.rs`: `InstallRequest`, `InstallPlan`, `InstallOperation`, managed
  manifests, conflict handling, and all install/update/uninstall execution.
- `catalog.rs`: shared editor, scope, category, conflict-mode, and source
  discovery contracts.
- `metadata.rs`: skill frontmatter, README attribution, origin/license/version
  data, and deterministic lockfile generation.
- `check.rs`: the repository-wide validation gate behind `stacc check`.
- `panel.rs` and `config.rs`: TUI orchestration and its persisted request
  defaults; the panel must call the same plan/execute path as the CLI.
- `bundle.rs` and `bootstrap.rs`: runtime payload resolution and binary
  bootstrap.
- `git_utils.rs`: the process boundary for Git status, HEAD, and remote lookups.
- `hook_selection.rs` and `selective.rs`: hook filtering and interactive
  per-file conflict decisions.

## Installer invariants

- Preserve the plan/execute split. `build_*_plan` functions validate and return
  operations; `execute_*` functions are the only mutation path.
- `--dry-run` may build and print plans but must not create, copy, remove,
  invoke plugin mutations, or update a manifest.
- Keep CLI and TUI behavior aligned by expressing changes in request and plan
  types instead of adding a second execution path.
- Treat `<target-root>/.stacc/manifest.json` as the ownership boundary.
  Managed update and uninstall must match manifest entries; never discover
  arbitrary editor folders and claim or remove them.
- Validate managed source paths before joining them: reject absolute, empty,
  parent, root, and platform-prefix components. Resolved destinations must stay
  under the selected editor target.
- Keep Codex plugin operations as planned command invocations. Do not vendor
  plugin payloads into the installer.

## Rust conventions

- Return `anyhow::Result` at I/O and command boundaries and add `Context` that
  identifies the failing path or operation.
- Use `Path`/`PathBuf` for filesystem boundaries; avoid string-built paths.
- Keep serialized and CLI-facing shapes explicit with Serde and Clap derives.
- Sort and deduplicate discovery output before planning or serialization so
  dry-run and lockfile output remain reproducible.
- Put focused unit tests in each module's `#[cfg(test)] mod tests`; add
  regression coverage for path, manifest, dry-run, or merge behavior changes.

## Validation

- Run targeted tests while iterating: `cargo test <test-name>`.
- Run the authoritative repository gate before handoff: `cargo run -- check`.
- For CLI planning changes, inspect a real plan with
  `cargo run -- install --dry-run --print-plan ...` and confirm no files changed.
