# CODEX PLUGIN ADAPTER

## SCOPE

This directory contains only configuration whose interface is Codex-specific.
Generic workflows imported from Codex repositories belong in core skills or a
domain stack.

## CONTENTS

- `skills/babysit-pr/` depends on Codex skill paths and Codex agent metadata.
- `plugins.json` is the opt-in Codex marketplace catalog. It references remote
  marketplaces; it does not vendor plugin payloads.

The public installer categories remain `codex-skills` and `codex-plugins`.
`codex-plugins` is global-only and requires an explicit `--codex-plugin`.

## CHANGE RULES

- Preserve `agents/openai.yaml`, scripts, references, and `LICENSE.txt` with the
  Codex skill package.
- Keep marketplace command arguments fixed and data-driven from `plugins.json`.
- Add a skill here only when its runtime or metadata is genuinely Codex-only.
- Put reusable PR, GitHub, or review guidance in a normal skill or stack.
- Update README attribution and refresh metadata after a package move.

## VALIDATION

```bash
cargo run -- install --editor codex --scope project --category codex-skills --dry-run --print-plan
cargo run -- install --editor codex --scope global --codex-plugin lazycodex --dry-run --print-plan
cargo run -- check
```
