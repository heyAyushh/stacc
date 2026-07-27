---
name: commit-push-pr
description: Review, commit, push, and create a pull request through the current branch's configured upstream. Use when you want to ship deliberate changes end-to-end without assuming a particular remote or base branch.
---

# Commit, Push, and Create PR

Review the changes and project checks before creating a commit, remote update, or
pull request.

## Preconditions

- GitHub CLI (`gh`) is installed and authenticated.
- The current branch has an intended upstream, or you have selected the remote
  to configure as its upstream.

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
   Run the relevant repository checks, then stage only the files that belong to
   this commit. Split unrelated work into separate commits.

3. **Commit and push**
   ```bash
   git add -- path/to/related-file
   git diff --cached
   git commit -m "<prefix>: <summary (imperative, concise)>"
   git push
   ```
   Use `git add -A` only when every pending item belongs in the commit.

4. **Create the PR**
   ```bash
   gh pr create --title "<prefix>: <summary>" --body "<summary and test plan>"
   ```
   Confirm the target/base branch shown by `gh` before submitting. If a PR
   already exists for the branch, update it rather than creating a duplicate.

## PR Content

Include a concise summary, the relevant validation performed, and any remaining
risks or follow-ups. Derive this from the reviewed diff and commits, not the
branch name alone.
