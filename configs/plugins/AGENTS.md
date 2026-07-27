# EDITOR PLUGIN PACKAGES

## PURPOSE

`configs/plugins/` contains adapters whose interface depends on one editor.
Reusable skills belong in `configs/skills/` or a domain stack, even when they
were imported from an editor's plugin repository.

## LAYOUT

| Editor | Contents | Installer categories |
| --- | --- | --- |
| `codex/` | Codex-only skills and the marketplace catalog | `codex-skills`, `codex-plugins` |
| `cursor/` | Cursor-only skills, agents, and hooks | `cursor-plugins` |

Use the editor's child `AGENTS.md` for package-specific rules.

## PLACEMENT TEST

Keep a package here only when removing the editor would break its interface:
editor SDK calls, hook manifests, transcript/state paths, editor agent formats,
or marketplace behavior. Source provenance alone is not enough.

Move a package to:

- `configs/skills/` when it is broadly useful and frequently installed.
- `configs/stacks/<domain>/` when it belongs to a focused workflow or platform.

Preserve the package directory, license, upstream URL, and pinned revision
during moves. Keep external category names stable because manifests and CLI
invocations use them even though physical sources live under `plugins/`.

## PROGRESSIVE DISCLOSURE

- Keep editor-wide ownership and install rules in the editor child guide.
- Keep package workflow in its `SKILL.md`.
- Keep detailed references and deterministic helpers beside that skill.
- Add deeper `AGENTS.md` files only for packages with their own runtime or
  maintenance seam.

## VALIDATION

Run the matching category with `--dry-run --print-plan`, refresh metadata, and
finish with `cargo run -- check`.
