# Orchestrate Skill Guide

This directory is the canonical source for the explicit `/orchestrate` Cursor
skill under `configs/plugins/cursor/`. The installed copy under
`.cursor/skills/orchestrate/` is a mirror; edit this package, then use the
repository refresh workflow rather than patching the mirror.

## Where to Look

- `SKILL.md`: trigger contract, roles, setup, and first-screen guidance.
- `references/dispatcher.md`: local one-shot kickoff behavior.
- `references/planner.md`: root and recursive planner loop.
- `references/spawning.md`: task dependencies, branches, prompts, and Slack.
- `references/handoffs.md`: structured results and merge-task protocol.
- `prompts/`: templates rendered for planners, workers, and verifiers.
- `scripts/cli.ts`: command surface and orchestration entry point.
- `scripts/core/`: plan execution, state, branches, handoffs, and Andon.
- `scripts/adapters/`: Cursor and Slack service boundaries.
- `scripts/schemas.ts`: runtime Zod contracts for plans and state.
- `schemas/`: generated JSON schemas; do not hand-edit them.
- `scripts/__tests__/`: behavior and recovery coverage.

## Local Commands

Run these from `scripts/`:

```bash
bun test
bun run typecheck
bun run lint
bun run generate-schemas
```

`bun run check` includes `biome check --write` and therefore changes formatting;
use the non-writing commands above for inspection. After changing Zod plan or
state contracts, run `bun run generate-schemas` and commit the generated schema
updates with the source change.

## Invariants

- Treat validated `plan.json`, `state.json`, handoff files, and recorded git
  branches as operational truth. Inspect `.orchestrate/` artifacts instead of
  reconstructing state from logs or Slack.
- Keep plan and state writes atomic and reject malformed data through the strict
  schemas. Recovery must converge from persisted identity and lineage.
- Preserve `startingRef` and `dependsOn` semantics. Verifiers reconcile against
  the actual branch reported by the target handoff unless the planner supplied
  an explicit override.
- Workers remain isolated: no sibling communication or shared mutable state.
  They commit and push their current branch, without renaming, merging, or
  rebasing it. Integration is a separately planned merge task.
- Structured handoffs are the only agent-to-planner result channel. Preserve
  failure sidecars and raw-output fallbacks so interrupted runs stay auditable.
- An active Andon pauses new spawns. Root polling owns Slack reactions;
  subplanners consume the persisted root state.
- Slack is visibility, not correctness. Keep status and comments inside the
  approved run thread; disk artifacts and git remain authoritative.
- Never place API keys, tokens, or other credentials in prompts, plans,
  handoffs, logs, or committed artifacts.

## Change Discipline

- Keep role-specific detail in the matching reference instead of expanding
  `SKILL.md`.
- Update templates only when the prompt contract changes; fix task intent in
  plan fields rather than patching a rendered prompt.
- Add focused tests for schema, recovery, branching, handoff, or adapter
  behavior when those boundaries change.
