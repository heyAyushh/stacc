---
name: rebase
description: Rebase the current branch with flexible target options. Use when you need to rebase on main, a specific branch, or a remote branch, with intelligent conflict handling.
---

# Rebase

Rebase the current branch.

## Arguments

- No arguments: rebase on local main
- `origin`: fetch origin, rebase on origin/main
- `<remote>/<branch>`: when `<remote>` is an existing Git remote, fetch it and rebase on that remote-tracking branch
- `branch`: rebase on local branch

## Steps

1. **Preflight the current branch**
   - Confirm this is a Git worktree and identify the current branch with `git branch --show-current`; stop for a detached `HEAD` unless the user explicitly chose it.
   - Run `git status --porcelain`; if it is not empty, stop and ask the user to commit, stash, or discard the changes. Do not stash or discard work automatically.
   - If the branch has an upstream, report its ahead/behind count with `git rev-list --left-right --count @{upstream}...HEAD`. Treat it as potentially published or shared and get explicit confirmation that rewriting and any required force-push are acceptable. If it has no upstream, state that publication cannot be determined.

2. **Resolve the target**
   - No args → target is `main`; verify it resolves to a commit.
   - Just `origin` → verify `origin` is a configured remote, fetch it, then target `origin/main`.
   - For another argument containing `/`, inspect its first path component with `git remote`. Fetch and use it as `<remote>/<branch>` only when that component is an existing remote; otherwise treat the whole argument as a local ref (for example, `feature/login`).
   - Verify the chosen target with `git rev-parse --verify <target>^{commit}` before rebasing.

3. **Fetch if needed**
   ```bash
   git fetch <remote>
   ```

4. **Rebase**
   ```bash
   git rebase <target>
   ```

5. **Handle conflicts** (if any)

6. **Continue until complete**

## Handling Conflicts

- BEFORE resolving any conflict, understand changes made to each conflicting file in the target branch
- For each conflicting file:
  ```bash
  git log -p -n 3 <target> -- <file>
  ```
- Determine the intended behavior from both changes, tests, and project conventions. Keep the correct behavior; do not mechanically preserve both edits when they conflict.
- After resolving each conflict:
  ```bash
  git add <file>
  git rebase --continue
  ```
- If a conflict is too complex or unclear, ask for guidance before proceeding
- At any point before completion, recover the pre-rebase state with:
  ```bash
  git rebase --abort
  ```
