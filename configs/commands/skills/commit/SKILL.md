---
name: commit
description: Commit reviewed local changes on the current branch using conventional commit messages. Use when you need to create one or more atomic commits without pushing.
---

# Commit Changes

Review the whole change set, then choose a single atomic commit or a small
series of atomic commits. Do not silently include unrelated work.

## Steps

1. **Assess all pending changes**
   ```bash
   git status --short
   git diff
   git diff --cached
   ```

2. **Choose the commit shape**
   - Use one commit when the implementation, tests, and docs serve one concern.
   - Split unrelated concerns into separate commits, keeping every intermediate
     commit buildable where practical.

3. **Stage and commit deliberately**
   ```bash
   git add -- path/to/related-file
   git diff --cached
   git commit -m "<prefix>: <summary (imperative, concise)>"
   ```
   Repeat step 3 for each concern. Use `git add -A` only after confirming every
   staged, unstaged, and untracked item belongs in the same commit.

## Commit Message Format

Use conventional commit prefixes:

- `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`,
  `style`, or `revert`

Example:
```bash
git add src/parser.rs tests/parser_test.rs
git diff --cached
git commit -m "fix: handle empty parser input"
```

## Notes

- This command only commits; it does not push or change branch-protection policy.
- Run the relevant project checks before committing when the repository defines them.
