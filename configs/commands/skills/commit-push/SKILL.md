---
name: commit-push
description: Review, commit, and push local changes through the current branch's configured upstream. Use when you want to ship a deliberate commit without assuming a particular remote.
---

# Commit and Push

Commit reviewed changes and push only through the branch's configured upstream.

## Steps

1. **Confirm branch and upstream**
   ```bash
   BRANCH="$(git branch --show-current)"
   UPSTREAM="$(git rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true)"
   test -n "$BRANCH" || { echo "Detached HEAD; create or check out a branch first"; exit 1; }
   test -n "$UPSTREAM" || { echo "No upstream configured; choose the intended remote, then run: git push -u <remote> $BRANCH"; exit 1; }
   case "$BRANCH" in main|master) echo "Direct pushes to $BRANCH are not allowed"; exit 1;; esac
   printf 'Branch: %s\\nUpstream: %s\\n' "$BRANCH" "$UPSTREAM"
   ```
   Do not push directly to protected branches such as `main` or `master`.

2. **Review and validate**
   ```bash
   git status --short
   git diff
   git diff --cached
   ```
   Run the relevant project checks documented by the repository before staging.

3. **Stage, commit, and push**
   ```bash
   git add -- path/to/related-file
   git diff --cached
   git commit -m "<prefix>: <summary (imperative, concise)>"
   git push
   ```
   Use `git add -A` only when every pending item belongs in this commit. Split
   unrelated work into separate commits instead of pushing it together.

## Notes

- `git push` uses the resolved upstream rather than assuming `origin`.
- Use conventional prefixes: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`,
  `build`, `ci`, `chore`, `style`, or `revert`.
