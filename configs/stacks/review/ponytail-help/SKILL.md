---
name: ponytail-help
description: >
  Quick-reference card for all ponytail modes, skills, and commands.
  One-shot display, not a persistent mode. Trigger: /ponytail-help,
  "ponytail help", "what ponytail commands", "how do I use ponytail".
---

# Ponytail Help

Display this reference card when invoked. One-shot, do NOT change mode,
write flag files, or persist anything.

## Levels

| Level | Invocation | What changes |
|-------|------------|--------------|
| **Lite** | Invoke `ponytail` with `lite` | Build what's asked, name the lazier alternative in one line. |
| **Full** | Invoke `ponytail` | The ladder enforced: YAGNI → stdlib → native → one line → minimum. |
| **Ultra** | Invoke `ponytail` with `ultra` | YAGNI extremist. Deletion before addition. Challenges requirements before building. |

Invocation syntax depends on the editor. Treat a level as applying to the
current request unless the host explicitly supports session state.

## Skills

| Skill | Invocation | What it does |
|-------|---------|--------------|
| **ponytail** | `ponytail` | Lazy mode itself. Simplest solution that works. |
| **ponytail-review** | `ponytail-review` | Over-engineering review: `L42: yagni: factory, one product. Inline.` |
| **ponytail-audit** | `ponytail-audit` | Whole-repo over-engineering audit: ranked list of what to delete. |
| **ponytail-debt** | `ponytail-debt` | Harvest `ponytail:` shortcut comments into a tracked ledger. |
| **ponytail-gain** | `ponytail-gain` | Explain the mode's boundaries; it does not estimate savings for this repo. |
| **ponytail-help** | `ponytail-help` | This card. |

Use your editor's installed-skill invocation mechanism; this stack does not
install a persistent Ponytail mode, configuration file, plugin marketplace, or
auto-update integration.
