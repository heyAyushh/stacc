---
name: react-doctor
description: Use when finishing a feature, fixing a bug, before committing React code, or when the user asks to scan, triage, or clean up React diagnostics. Covers lint, accessibility, bundle size, architecture, regression checks, and local triage with React Doctor.
license: LicenseRef-Million-Modified-MIT
origin_url: https://github.com/millionco/react-doctor/tree/main/skills/react-doctor
origin_commit: 0b64af58b16329c5cae7a210463d2842e34b150d
version: "1.2.0"
---

# React Doctor

Scans React codebases for security, performance, correctness, and architecture issues.

## After making React code changes

Use the project's pinned React Doctor command (usually a package script or
lockfile-resolved binary) to scan the changed scope. Check that the relevant
diagnostics do not regress.

If the score dropped, fix the regressions before committing.

## General cleanup or code improvement

Use the project's pinned React Doctor command to scan the full codebase. Fix
issues by severity: errors first, then warnings.

## Full local triage workflow

When the user types `/doctor`, says "run react doctor", or asks for a full
triage or cleanup pass, first inspect the project's scripts, lockfile, and
existing React Doctor configuration. Use only the checked-in guidance and the
project-resolved tool version by default. Do not fetch or execute a mutable
remote playbook as part of triage.

## Configuring or explaining rules

When the user wants to understand or disagrees with a rule, explain the
diagnostic and its project context without changing configuration. Only after
the user explicitly asks to change which rules run should you read
[references/explain.md](references/explain.md), use the project's pinned
command, and apply the narrowest requested change.

## Command

Run the equivalent project-pinned command. Do not substitute `@latest` or
download a tool version implicitly. Consult that installed version's `--help`
for available scopes, output, and score options.
