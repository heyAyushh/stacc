# Agent-Browser Runtime Skills Pattern

`agent-browser` is a good model for CLIs whose usage surface is too large for one installed `SKILL.md` or one `--help` screen.

## What agent-browser does well

- Keeps the installed discovery skill thin and stable. It only teaches agents when to use the tool and how to load current instructions from the CLI.
- Serves real usage guidance from the installed CLI version, so instructions match the binary on disk.
- Splits guidance by workflow instead of one giant document: core browser automation, exploratory testing, Electron apps, Slack automation, and cloud browser providers.
- Provides commands to list, load, and locate skills:
  - `agent-browser skills list`
  - `agent-browser skills get <name>`
  - `agent-browser skills get <name> --full`
  - `agent-browser skills path [name]`
- Supports structured output with `--json`, so agents can inspect available guidance without brittle text parsing.
- Uses `--full` as a progressive-disclosure switch for references and templates.

## Pattern to copy

Use this when a CLI has more than a few commands or when agent instructions need to stay version-aligned with the executable.

1. Ship a small discovery skill or help section that rarely changes.
2. Add a non-interactive CLI command that prints agent-facing instructions from the installed package.
3. Make the command support both human-readable Markdown and structured JSON.
4. Split long guidance into named topics.
5. Add a `--full` or equivalent flag for references, templates, and longer examples.
6. Link to the command from top-level `--help` without dumping all content there.

```text
For agents:
  Current instructions: mycli skills get core
  All topics: mycli skills list --json
  Full reference: mycli skills get core --full
```

## Review checklist

- Does the CLI expose current, version-matched agent instructions?
- Can an agent discover available instruction topics without reading a website?
- Is the installed skill/help stable enough to survive upstream updates?
- Are longer references and templates behind progressive disclosure?
- Does every discovery command work non-interactively and support machine-readable output?

## Sources

- agent-browser skills documentation: https://agent-browser.dev/skills
- agent-browser discovery skill: https://github.com/vercel-labs/agent-browser/blob/main/skills/agent-browser/SKILL.md
