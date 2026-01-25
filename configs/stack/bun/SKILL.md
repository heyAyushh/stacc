---
name: bun
description: Bun runtime and modern JavaScript backend practices, including Bun-native tooling (bun run/test/build) and PostgreSQL usage guidance. Use when building or reviewing Bun-based services, APIs, scripts, or when Bun-specific workflow questions arise.
---

# Bun Stack Skill

## Default guidance

Apply the rule files in this folder as the source of truth for Bun and PostgreSQL usage.

## Workflow (use this order)

1. Clarify scope: service/API/script/task runner.
2. Prefer Bun-native tooling (`bun install`, `bun run`, `bun test`, `bun build`).
3. Use Bun-native APIs (`Bun.serve`, `Bun.file`, SQLite/Redis) where applicable.
4. Design module boundaries and error handling early.
5. Add tests with `bun test` and include coverage for edge cases.
6. Validate deployment target and build output.

## Review Checklist

- Bun-native tooling used; no unnecessary Node tooling.
- Modern JS/TS patterns (ESM, async/await, private fields) applied.
- Error handling is centralized and user-facing errors are clear.
- IO paths use Bun APIs for performance.
- Tests present and runnable with `bun test`.

## Local Resources

- `bun.mdc` for Bun runtime, tooling, and modern JS/TS backend best practices.
- `postgresql.mdc` for PostgreSQL SQL conventions, performance, and anti-patterns.
