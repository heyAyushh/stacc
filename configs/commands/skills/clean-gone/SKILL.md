---
name: clean-gone
description: Safely clean up local branches whose tracked remote branch is gone. Use when you want to review stale branches and associated worktrees after remote branches or PRs are deleted.
---

# Clean Gone Branches

Review every candidate before removing anything. `[gone]` means the tracked
remote ref is absent; it does not mean the local branch or worktree is safe to
discard.

## Steps

1. **Refresh and list candidates**
   ```bash
   git fetch --prune
   git branch -vv
   ```
   Record only branches whose upstream is shown as `[gone]`.

2. **Inspect each candidate and any linked worktree**
   ```bash
   BRANCH="<gone-branch>"
   git log --oneline --decorate --max-count=20 "$BRANCH"
   git worktree list --porcelain
   ```
   If a worktree is linked to the branch, run `git -C <worktree-path> status
   --short`. Stop if it has changes, or if the branch contains work worth
   keeping.

3. **Confirm, then remove one clean worktree and branch at a time**
   ```bash
   git worktree remove "<clean-worktree-path>"
   git branch -d "$BRANCH"
   ```
   Ask for confirmation of the selected paths before running these commands.
   Do not use `--force` by default. If either command refuses, report why and
   require an explicit decision before considering a destructive override.

## Expected Outcome

- Report the candidates reviewed and the branches/worktrees actually removed.
- If no branches are marked `[gone]`, report that no cleanup was needed.
