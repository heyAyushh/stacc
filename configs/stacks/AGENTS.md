# STACK PACKAGE GUIDE

## PURPOSE

`configs/stacks/` is the canonical source for framework- and language-specific
bundles installed by the `stack` category. Each immediate child directory is one
selectable stack.

## PACKAGE BOUNDARY

- `bun/`, `databases/`, `ios/`, `nextjs/`, `react-native/`, `rust/`, `solana/`,
  `turborepo/`, and `typescript/` are the selectable units.
- The installer discovers only those immediate children and copies a selected
  directory wholesale into the destination skill root.
- Nested skills, rules, references, scripts, metadata, and `AGENTS.md` files are
  payload belonging to their top-level stack. They are not separate `--stack`
  values.
- Preserve stable relative paths within a stack because installed consumers can
  refer to those paths.

## WHERE TO WORK

| Change | Location |
| --- | --- |
| Stack-wide behavior or entry instructions | `<stack>/SKILL.md` |
| A nested reusable capability | `<stack>/<skill>/SKILL.md` |
| Detailed background or command catalogs | The nearest `references/` |
| Repeatable maintenance or validation | The nearest `scripts/` |
| Editor-specific rule payload | The stack's `rules/` or `.mdc` files |
| Package-specific contributor guidance | The nearest child `AGENTS.md` |

The nearest child `AGENTS.md` is authoritative for its subtree. Existing large
leaf guides under Next.js and React Native are shipped domain payload for those
packages; they are intentionally scoped away from repository-level routing and
should not be copied into this guide.

## PROGRESSIVE DISCLOSURE

- Keep `SKILL.md` focused on triggering, routing, and the first useful workflow.
- Move long examples, API catalogs, background material, and refresh procedures
  into linked `references/` or `scripts/`.
- Add a child `AGENTS.md` only when a subtree has distinct maintenance rules;
  do not create one merely to repeat this file.
- Edit canonical files here, not installed mirrors under `.agents/`, `.claude/`,
  `.codex/`, `.cursor/`, or `.opencode/`.

## PROVENANCE

- Preserve upstream license files and provenance metadata when importing content.
- Record copied or adapted packages in the README attribution table with source
  URL, license, and pinned revision when available.
- Keep `configs/metadata/skills.lock.json` consistent with the canonical package
  so source-freshness checks remain meaningful.

## VALIDATION

```bash
cargo run -- install --editor codex --scope project --category stack --stack <name> --dry-run --print-plan
cargo run -- check
```

Confirm the dry-run selects the top-level stack once, retains nested payload
paths, and does not emit writes before running the full repository gate.
