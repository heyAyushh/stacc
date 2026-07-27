---
name: react-doctor
description: Use when finishing a feature, fixing a bug, before committing React code, or when the user asks to scan, triage, or clean up React diagnostics. Covers lint, accessibility, bundle size, architecture, regression checks, and local triage with React Doctor.
license: LicenseRef-Million-Modified-MIT
origin_url: https://github.com/millionco/react-doctor/tree/main/skills/react-doctor
origin_commit: 0b64af58b16329c5cae7a210463d2842e34b150d
version: "1.2.0"
---

# React Doctor

Scans React codebases for security, performance, correctness, and architecture issues. Outputs a 0-100 health score.

## After making React code changes

Run `npx react-doctor@latest --verbose --scope changed` and check the score did not regress.

If the score dropped, fix the regressions before committing.

## General cleanup or code improvement

Run `npx react-doctor@latest --verbose` to scan the full codebase. Fix issues by severity: errors first, then warnings.

## Full local triage workflow

When the user types `/doctor`, says "run react doctor", or asks for a full triage or cleanup pass, fetch the canonical local-triage playbook and follow every applicable step:

```bash
curl --fail --silent --show-error \
  --header 'Cache-Control: no-cache' \
  https://www.react.doctor/prompts/react-doctor-agent.md
```

The playbook is the single source of truth for scan, filter, triage, fix, and validate loops. Pair it with the matching per-rule prompts at `https://www.react.doctor/prompts/rules/<plugin>/<rule>.md` when a rule needs a canonical fix recipe.

## Configuring or explaining rules

When the user wants to understand a rule, disagrees with one, or wants to disable or tune which rules run, read [references/explain.md](references/explain.md) and follow it. Start with `npx react-doctor@latest rules explain <rule>`, then apply the narrowest control via `npx react-doctor@latest rules disable|set|category|ignore-tag ...`, which edits `doctor.config.*` or `package.json#reactDoctor`.

## Command

```bash
npx react-doctor@latest --verbose --scope changed
```

| Flag | Purpose |
| --- | --- |
| `.` | Scan current directory |
| `--verbose` | Show affected files and line numbers per rule |
| `--scope changed` | Only report issues introduced vs the base branch |
| `--scope lines` | Only report issues on the changed lines |
| `--score` | Output only the numeric score |
