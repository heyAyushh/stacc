# stacc-cli-sync-model - Work Plan

## TL;DR (For humans)
<!-- Fill this LAST, after the detailed plan below is written, so it summarizes the REAL plan. -->
<!-- Plain English for a non-engineer: NO file paths, NO todo numbers, NO wave/agent/tool names. -->

**What you'll get:** A clear map of what `stacc`, `install.sh`, `sync`, `update`, `uninstall`, and `sync-metadata` currently do, plus the safest model for GitHub-backed external skill updates.

**Why this approach:** The current code intentionally separates deployed local stacc payloads from upstream attribution metadata. Keeping that boundary prevents stacc from taking ownership of arbitrary user-installed skills.

**What it will NOT do:** It will not change product code, global skill folders, or any user-installed skill payloads in this turn.

**Effort:** Quick
**Risk:** Low - explanation and plan only; no product writes.
**Decisions to sanity-check:** Whether future GitHub updates should be a new source-refresh command or an extension of metadata sync.

Your next move: Approve a follow-up implementation only if GitHub-backed external source refresh should be added. Full execution detail follows below.

---

> TL;DR (machine): Quick/low-risk explanation; no product changes; future implementation should add explicit source refresh before managed deploy.

## Scope
### Must have
- Explain the current CLI.
- Explain what `install.sh` is.
- Explain what `sync` does and where it reads from.
- Explain how externally sourced skills can be kept updated if GitHub is upstream.

### Must NOT have (guardrails, anti-slop, scope boundaries)
- No product code edits.
- No writes outside the project root.
- No changes to `.git`, `.env`, credential files, or user global skill folders.
- No claim that `sync` fetches GitHub payloads.

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: none; no product code change.
- Evidence: code reads, README/public GitHub comparison, and dry-run CLI commands:
  - `cargo run -- status --json`
  - `cargo run -- install --editor codex --scope project --category skills --dry-run --print-plan`
  - `cargo run -- sync --editor codex --scope project --dry-run --print-plan`
  - `cargo run -- sync --editor codex --scope project --skill ultragoal --dry-run --print-plan`
  - `cargo run -- update --editor codex --scope project --skill ultragoal --dry-run --print-plan`
  - `cargo run -- sync-metadata --dry-run --json`

## Execution strategy
### Parallel execution waves
> Target 5-8 todos per wave. Fewer than 3 (except the final) means you under-split.
- Wave 1: Read command definitions, installers, manifest sync/update logic, metadata sync logic, README docs, public GitHub README.
- Wave 2: Run dry-run surfaces to confirm the observed behavior.
- Wave 3: Record findings and answer.

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1 | none | 2, 3 | 2 after broad reads |
| 2 | 1 | 3 | independent dry-runs |
| 3 | 1, 2 | final | none |

## Todos
> Implementation + Test = ONE todo. Never separate.
<!-- APPEND TASK BATCHES BELOW THIS LINE WITH edit/apply_patch - never rewrite the headers above. -->
- [x] 1. Verify CLI and shell bootstrap behavior
  What to do / Must NOT do: Read `src/main.rs` and `install.sh`; do not infer from memory.
  Parallelization: Wave 1 | Blocked by: none | Blocks: 3
  References (executor has NO interview context - be exhaustive): `src/main.rs:58`, `src/main.rs:365`, `install.sh:32`, `install.sh:335`
  Acceptance criteria (agent-executable): Identified command list and bootstrap forwarding path.
  QA scenarios (name the exact tool + invocation): `cargo run -- status --json`
  Commit: N

- [x] 2. Verify sync/update/metadata source behavior
  What to do / Must NOT do: Read planner code and run dry-runs; do not claim GitHub payload refresh exists.
  Parallelization: Wave 1/2 | Blocked by: none | Blocks: 3
  References (executor has NO interview context - be exhaustive): `src/install.rs:719`, `src/install.rs:801`, `src/install.rs:885`, `src/metadata.rs:153`, `src/metadata.rs:542`
  Acceptance criteria (agent-executable): Dry-run proves sync backfill and update manifest gate.
  QA scenarios (name the exact tool + invocation): `cargo run -- sync --editor codex --scope project --skill ultragoal --dry-run --print-plan`; `cargo run -- update --editor codex --scope project --skill ultragoal --dry-run --print-plan`
  Commit: N

- [x] 3. Produce answer and safe future design
  What to do / Must NOT do: Answer current behavior and propose explicit GitHub source-refresh model; do not implement.
  Parallelization: Wave 3 | Blocked by: 1, 2 | Blocks: final
  References (executor has NO interview context - be exhaustive): `README.md:320`, `README.md:388`
  Acceptance criteria (agent-executable): Final answer distinguishes install/sync/update/sync-metadata and external skills.
  QA scenarios (name the exact tool + invocation): Review final answer against command output.
  Commit: N

## Final verification wave
> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.
- [x] F1. Plan compliance audit
- [x] F2. Code quality review
- [x] F3. Real manual QA
- [x] F4. Scope fidelity

## Commit strategy
No commit. Only `.omo` scratch artifacts changed for this planning/explanation turn.

## Success criteria
- User gets an accurate answer grounded in current code, README, public GitHub, and dry-run behavior.
- Product code remains untouched.
- Future implementation path is clear and does not broaden stacc ownership over arbitrary local skill folders.
