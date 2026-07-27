---
slug: stacc-cli-sync-model
status: drafting
intent: clear
pending-action: answer user with current model and recommended next design
approach: Explain current behavior from code/docs/dry-runs; propose a safe GitHub-source update model without changing product files.
---

# Draft: stacc-cli-sync-model

## Components (topology ledger)
<!-- Lock the SHAPE before depth. One row per top-level component that can succeed or fail independently. -->
<!-- id | outcome (one line) | status: active|deferred | evidence path -->
CLI | Rust binary owns status/install/sync/update/uninstall/sync-metadata/bootstrap/check | active | src/main.rs:58-75, src/main.rs:365-448
Shell bootstrap | install.sh finds/installs/runs stacc and adapts legacy flags | active | install.sh:1-67, install.sh:335-403
Install sources | install copies from checked-out/bundled configs into editor targets and writes manifest | active | src/install.rs:594-614, README.md:388-394, dry-run install output
Manifest sync | sync only backfills already-installed stacc packages into manifest; it does not copy missing packages | active | src/install.rs:719-755, dry-run sync output
Managed update/uninstall | update/uninstall require stacc manifest entries and copy/remove only recorded managed destinations | active | src/install.rs:885-930, dry-run update error
Metadata sync | sync-metadata audits local skill metadata and optional GitHub origin heads; it does not update payload files | active | src/metadata.rs:105-160, src/metadata.rs:542-553
GitHub-backed external updates | Not currently implemented as payload refresh; should be a separate source-import/update workflow | deferred | README.md:320-395, src/metadata.rs:153-160

## Open assumptions (announced defaults)
<!-- Record any default you adopt instead of asking, so the user can veto it at the gate. -->
<!-- assumption | adopted default | rationale | reversible? -->
User wants operating model, not immediate code change | Answer plus design recommendation | The ask is "what does" and "how can"; no edit request in this turn | yes
Stacc should not claim arbitrary local skills | Keep manifest boundary | Existing update/uninstall safety is explicitly ownership-based | yes, but only with explicit adoption command

## Findings (cited - path:lines)
- `stacc` CLI subcommands are `status`, `install`, `update`, `uninstall`, `sync`, `sync-metadata`, `bootstrap`, and `check` (`src/main.rs:58-75`).
- `install`, `update`/`uninstall`, `sync`, and `sync-metadata` dispatch to typed Rust planners/executors (`src/main.rs:365-448`).
- `install.sh` is documented and implemented as bootstrap plus legacy adapter. It handles direct stacc invocations and translates old flags (`install.sh:32-67`, `install.sh:335-403`).
- `install` for a skill category copies from `configs/skills/...` into the target editor skill root and updates `<target-root>/.stacc/manifest.json`; dry-run confirmed planned copies from `configs/skills/*` to `.codex/skills/*`.
- `sync` scans stacc source packages and only records destinations that already contain `SKILL.md` (`src/install.rs:719-755`). Dry-run with no installed `.codex/skills/ultragoal` produced `skip ... installed stacc skill not found`.
- `update` requires the manifest. Dry-run without `.codex/.stacc/manifest.json` failed with `stacc manifest missing ... install the skill or plugin with stacc first`.
- Skill metadata discovers stacc sources under `configs/skills`, `configs/codex-skills/skills`, `configs/cursor-plugins/skills`, and `configs/stacks` (`src/metadata.rs:542-553`).
- GitHub origin heads are only refreshed when `sync-metadata --refresh-origin` is used (`src/metadata.rs:153-160`); this produces metadata, not payload updates.
- README attributions record external source URLs/licenses for vendored skills and plugins (`README.md:320-395`).

## Decisions (with rationale)
- Treat the stacc repo/bundle as the deployed source of truth for `install`, `sync`, `update`, and `uninstall`.
- Treat external GitHub repos as upstream source evidence, not the current deployed payload source, until stacc gets an explicit source-refresh workflow.
- Keep update/uninstall manifest-gated so arbitrary user-installed local skills are not touched.
- Recommended future model: add a source manifest plus explicit `source update`/`import` path that fetches GitHub into a temp area, verifies path/license/metadata, updates vendored `configs/...`, then lets existing managed `update` deploy the new local stacc copy.

## Scope IN
- Explain current CLI/install.sh/sync/update model.
- Explain where synced skills come from.
- Explain what happens to skills added outside stacc.
- Recommend safe way to keep GitHub-sourced skills updated.

## Scope OUT (Must NOT have)
- No product code edits in this turn.
- No writes outside repo root.
- No modification of user global skills.
- No destructive operations.

## Open questions
- Should GitHub-backed source refresh be implemented as `stacc source update`, `stacc import`, or an extension of `sync-metadata`?
- Should adoption of arbitrary existing local skills require an explicit `adopt` command with source URL and license metadata?

## Approval gate
status: ready-to-answer
<!-- When exploration is exhausted and unknowns are answered, set status: awaiting-approval. -->
<!-- That durable record is the loop guard: on a later turn read it and resume at the gate instead of re-running exploration. -->
