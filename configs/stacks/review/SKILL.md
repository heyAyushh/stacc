---
name: review
description: Route code cleanup, maintainability, React diagnostics, and over-engineering review to focused review modes. Use when auditing code quality or asking for a particular review lens.
license: MIT
---

# Review stack

Choose one review interface:

- Branch-local cleanup edits: `deslop`.
- Explicit deep structural, maintainability review: `thermo-nuclear-code-quality-review`.
- React diagnostics: `react-doctor`.
- Surgical coding defaults: `karpathy-guidelines`.
- Implementation constraint: `ponytail`.
- Non-mutating over-engineering review: `ponytail-review` (diff) or
  `ponytail-audit` (whole repository).
- Ponytail utilities: `ponytail-debt`, `ponytail-gain`, or `ponytail-help`.

The Ponytail entrypoints have distinct triggers; keep them separate. Read only
the selected child `SKILL.md`.
