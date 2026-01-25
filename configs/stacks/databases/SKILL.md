---
name: db
description: Database (PostgreSQL) SQL best practices for clean, performant, and maintainable queries. Use when writing or reviewing SQL, schema changes, or database-related guidance.
---

# Database Stack Skill

## Default guidance

Use the PostgreSQL rule file in this folder as the authoritative reference.

## Workflow (use this order)

1. Confirm schema context and target tables/columns.
2. Draft query with explicit columns and clear aliases.
3. Prefer CTEs for readability and reuse.
4. Avoid `NOT IN`; use `NOT EXISTS` or `LEFT JOIN ... IS NULL`.
5. Run `explain analyze` and address sequential scans.
6. Add or adjust indexes if query plan needs them.

## Review Checklist

- Naming uses `snake_case`; SQL keywords are lowercase.
- Explicit joins used; no implicit joins.
- `select *` avoided; columns are explicit.
- `NOT IN` avoided with `NULL`-safe alternatives.
- Query plan checked with `explain analyze`.

## Local Resources

- `postgresql.mdc` for naming, formatting, query patterns, and performance guidance.
