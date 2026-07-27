# Commit Changes

Commit local changes following conventional commit message format. Supports intelligent splitting of large change sets into multiple atomic commits.

## Overview

This command stages and commits changes with properly formatted messages. For large or complex change sets, it analyzes changes and creates multiple atomic commits grouped by logical concern.

## Preconditions

- Modified or untracked files exist
- Working directory is inside a git repository

## Steps

1. **Assess the change set**
   ```bash
   git status
   git diff --stat
   ```

2. **Determine commit strategy**
   - **Single commit**: All changes are related to one concern (< 10 files, single feature/fix)
   - **Multiple commits**: Changes span multiple concerns (different features, unrelated fixes, mixed refactoring)

3. **For single commit**: Stage all and commit
   ```bash
   git add -A
   git commit -m "<prefix>: <summary>"
   ```

4. **For multiple commits**: Group and commit separately (see Multi-Commit Workflow below)

## Commit Message Format

Use conventional commit prefixes:
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring (no behavior change)
- `perf`: Performance improvement
- `test`: Tests
- `docs`: Documentation
- `build`: Build system or dependencies
- `ci`: CI/CD configuration
- `chore`: Maintenance tasks
- `style`: Code style (formatting, semicolons)
- `revert`: Revert previous commit

### Message Structure

```
<prefix>: <summary (imperative, ~50 chars)>

[optional body with bullet points]

[optional footer: Refs #issue, BREAKING CHANGE, Co-Authored-By]
```

## Multi-Commit Workflow

When changes span multiple concerns, split into atomic commits:

### 1. Analyze and group changes

```bash
# See all changes
git status

# Group mentally or list by concern:
# - Feature A: src/feature-a.ts, tests/feature-a.test.ts
# - Bug fix: src/utils.ts
# - Docs: README.md, docs/api.md
```

### 2. Commit each group separately

```bash
# Commit group 1
git add src/feature-a.ts tests/feature-a.test.ts
git commit -m "feat: add feature A with validation"

# Commit group 2
git add src/utils.ts
git commit -m "fix: handle null input in parseData"

# Commit group 3
git add README.md docs/api.md
git commit -m "docs: update API documentation"
```

### Grouping Heuristics

| Concern | Files to group together |
|---------|------------------------|
| Feature | Implementation + tests + types for that feature |
| Bug fix | Fix + test that reproduces the bug |
| Refactor | All files affected by the refactor |
| Docs | Related documentation files |
| Config | Build/CI/config files that change together |
| Dependencies | package.json + lockfile |

### When to Split

Split changes into multiple commits when:
- Changes address more than one issue or feature
- Unrelated files are modified (e.g., a fix + a feature + docs)
- The diff would be hard to review as a single commit
- Different prefixes would apply to different changes
- Rolling back part of the changes might be needed later

### When NOT to Split

Keep as single commit when:
- All changes serve one purpose
- Files are tightly coupled (implementation + its tests)
- Splitting would create broken intermediate states

## Examples

### Single commit (simple case)

```bash
git add -A
git commit -m "fix: prevent crash on empty input array"
```

### Multiple commits (complex change set)

```bash
# Commit 1: New feature
git add src/auth/ tests/auth/
git commit -m "feat: add OAuth2 login flow"

# Commit 2: Unrelated fix discovered during development
git add src/utils/string.ts
git commit -m "fix: escape special chars in sanitizeInput"

# Commit 3: Documentation
git add docs/authentication.md
git commit -m "docs: add OAuth2 setup guide"

# Commit 4: Dependency update needed for feature
git add package.json package-lock.json
git commit -m "build: add oauth2-client dependency"
```

## Notes

- Always review diffs before committing (`git diff`, `git diff --cached`)
- Each commit should leave the codebase in a working state
- Commit messages should explain *why*, not just *what*
- This command does NOT push to remote or enforce branch policies
