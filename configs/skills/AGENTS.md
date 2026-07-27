# SKILLS KNOWLEDGE BASE

## OVERVIEW
`configs/skills` is the small, frequently used editor-neutral core installed by
the `skills` category. Focused domains live in `configs/stacks/`; editor
adapters live in `configs/plugins/`.

## CATEGORY MAP
| Role | Skills |
|---|---|
| Discovery and execution | `find-skills`, `diagnose`, `tdd` |
| Browser and desktop surfaces | `agent-browser`, `gui-automation` |
| Coordination | `handoff`, `ultragoal`, `using-git-worktrees` |
| Skill maintenance | `skill-creator`, `writing-great-skills` |

Adding a core skill requires evidence that it is broadly applicable and used
often enough to justify default installation. Otherwise place it in the
smallest matching stack.

## PROGRESSIVE DISCLOSURE
- `SKILL.md` is the routing surface: frontmatter, trigger description, core workflow, and the smallest useful command set.
- Put bulky material in `references/`: command catalogs, design rubrics, examples, API details, benchmark notes.
- Put deterministic helpers in `scripts/`; document inputs, outputs, and validation next to the script.
- When a `SKILL.md` becomes mostly catalogs, examples, or background material, split that material out instead of expanding the first screen.
- Reference only the files needed for the current workflow; do not require agents to load every reference up front.
- Prefer one skill per invocation mode. If a family has distinct triggers (`ponytail`, `ponytail-review`, `ponytail-audit`), keep separate folders instead of one branching prompt.
- Add a child `AGENTS.md` only for a genuinely distinct maintenance boundary, not to restate this guide.
- Do not hide external provenance in references. README attribution and `configs/metadata/skills.lock.json` must still identify source URL, license, and commit.

## IMPORT RULES
- Imported external commands are documentation, not instructions to run during import.
- Preserve upstream license as `LICENSE.txt` when a license exists.
- If upstream frontmatter uses unsupported top-level keys, move them under `metadata` and mention that adaptation in README attribution.
- After adding or changing skills, run `cargo run -- sync-metadata --refresh-origin` and `cargo run -- check`.
