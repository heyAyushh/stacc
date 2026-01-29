---
name: commit
description: Intelligent git commit workflow that analyzes changes and creates well-structured commits. Automatically determines whether to create a single commit or split large change sets into multiple atomic commits. Use when committing code changes, staging files, or when you have a mix of unrelated modifications that should be separate commits.
---

# Commit Skill

Create well-structured git commits with intelligent change analysis. Handles both simple single-file changes and complex multi-concern change sets.

## Workflow

### 1. Assess Changes

```bash
git status
git diff --stat
git diff  # Review actual changes
```

### 2. Decide: Single or Multiple Commits?

**Single commit** when:
- All changes serve one purpose
- < 10 files, single feature/fix/refactor
- Files are tightly coupled

**Multiple commits** when:
- Changes span different concerns (feature + fix + docs)
- Different conventional commit prefixes apply
- Changes are logically independent
- Large change set (> 15 files across different areas)

### 3. Execute

#### Single Commit
```bash
git add <files>  # or git add -A for all
git commit -m "<prefix>: <summary>"
```

#### Multiple Commits
Group by concern, commit each group:
```bash
# Group 1
git add src/feature.ts tests/feature.test.ts
git commit -m "feat: add user validation"

# Group 2
git add src/utils.ts
git commit -m "fix: handle edge case in parser"
```

## Commit Message Format

```
<prefix>: <summary (imperative, ~50 chars)>

[optional body]

[optional: Refs #issue, Co-Authored-By]
```

### Prefixes

| Prefix | Use for |
|--------|---------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code restructuring (no behavior change) |
| `perf` | Performance improvement |
| `test` | Adding/updating tests |
| `docs` | Documentation |
| `build` | Build system, dependencies |
| `ci` | CI/CD changes |
| `chore` | Maintenance, tooling |
| `style` | Formatting only |
| `revert` | Reverting commits |

## Grouping Heuristics

| Concern | Group together |
|---------|----------------|
| Feature | Implementation + tests + types |
| Bug fix | Fix + regression test |
| Refactor | All affected files |
| Docs | Related .md files |
| Config | Config files that change together |
| Deps | package.json + lockfile |

## Examples

### Simple: One Bug Fix
```bash
git add src/parser.ts
git commit -m "fix: prevent null pointer on empty input"
```

### Complex: Feature + Unrelated Fix + Docs
```bash
# Analyze
git status
# Shows: src/auth.ts, src/auth.test.ts, src/utils.ts, README.md

# Commit 1: The feature
git add src/auth.ts src/auth.test.ts
git commit -m "feat: add token refresh flow"

# Commit 2: Unrelated fix found during development
git add src/utils.ts
git commit -m "fix: sanitize user input in formatName"

# Commit 3: Documentation
git add README.md
git commit -m "docs: add authentication setup guide"
```

### New Files + Modified Files
```bash
# New skill files
git add configs/skills/commit/
git commit -m "feat: add commit skill with multi-commit support"

# Modified existing command
git add configs/commands/commit.md
git commit -m "docs: expand commit command with grouping guidance"
```

## Best Practices

- Each commit should leave codebase in working state
- Write message explaining *why*, not just *what*
- Review diffs before committing
- Stage specific files, not blindly `git add -A`
- Keep commits atomic and focused
