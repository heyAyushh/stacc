# CURSOR PLUGIN ADAPTER

## SCOPE

This directory contains only packages coupled to Cursor interfaces. Imported
but editor-neutral skills belong in core skills or a domain stack.

## CONTENTS

- `skills/continual-learning/` works with Cursor transcripts and hook state.
- `skills/orchestrate/` uses Cursor cloud-agent APIs and has its own
  `AGENTS.md`.
- `agents/agents-memory-updater.md` and `hooks/continual-learning/` form the
  continual-learning adapter and move together.

The public installer category remains `cursor-plugins`; the physical source
path is internal.

## CHANGE RULES

- Preserve hook manifests, `CURSOR_PLUGIN_ROOT` behavior, transcript paths, and
  agent prompt formats as one module.
- Keep generic review, learning, CLI, and summary skills in their domain stack.
- Add a package here only when its interface names Cursor state, SDKs, hooks, or
  agent formats.
- Update README attribution and refresh metadata after a package move.

## VALIDATION

```bash
cargo run -- install --editor cursor --scope project --category cursor-plugins --dry-run --print-plan
cargo run -- install --editor cursor --scope project --category hooks --hook continual-learning --dry-run --print-plan
cargo run -- check
```
