# Create command

## Overview

Create a new command file in `configs/commands/` that matches this repo's style and structure.
If the command accepts input, define how it should simplify the codebase or a specified file.

## Steps

1. Choose a concise kebab-case name (e.g. `create-command`).
2. Add a new file at `configs/commands/<name>.md`.
3. Use a clear title line and short, actionable sections.
4. Keep instructions minimal, direct, and consistent with existing commands.
5. If you need to reference files or tools, use backticks for paths and tool names.
6. If input is provided, state how to simplify the target (remove redundancy, align with repo style).

## Template

```md
# <Command name>

## Overview

<One or two sentences describing the intent and scope.>

## Steps

1. <Step one, imperative>
2. <Step two, imperative>
3. <Step three, imperative>

## Notes (optional)

- <Any constraints, assumptions, or out-of-scope items>
```

## Install locally (optional)

Copy the new command into your project or global tool folder:

```bash
# Cursor (project)
cp configs/commands/<name>.md .cursor/commands/

# Claude Code (project)
cp configs/commands/<name>.md .claude/commands/
```
