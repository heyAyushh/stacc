# SKILLS KNOWLEDGE BASE

## OVERVIEW
`configs/skills` holds editor-neutral skills installed by the `skills` category. Source-specific editor packages live elsewhere: `configs/cursor-plugins/` and `configs/codex-skills/`.

## CATEGORY MAP
| Category | Skills | Notes |
|---|---|---|
| Authoring + maintenance | `skill-creator`, `writing-great-skills`, `find-skills`, `mcp-builder`, `changelog-generator` | Create, package, discover, or summarize config assets. |
| Engineering workflow | `diagnose`, `tdd`, `prototype`, `to-prd`, `to-issues`, `triage`, `zoom-out`, `setup-matt-pocock-skills` | Process skills. Keep references close to the skill folder. |
| Communication + review modes | `caveman`, `ponytail*`, `karpathy-guidelines`, `bash-expert` | Behavior modes and review lenses. Ponytail is a family; do not merge the six entrypoints. |
| Frontend + product craft | `frontend-design`, `hallmark`, `brandkit`, `stitch-skill`, `redesign-skill`, `imagegen-*`, `taste-skill*`, `minimalist-skill`, `brutalist-skill`, `soft-skill` | Design/UI guidance. Prefer references for long visual rules and examples. |
| Platform + runtime | Expo skills, `building-native-ui`, `native-data-fetching`, `agent-browser`, `gui-automation`, `audio-math-haptics`, `add-app-clip` | Tool/runtime skills. Keep install commands informational unless user explicitly asks to run them. |
| Local conventions | `using-git-worktrees`, `ultragoal`, repo-specific imports | Keep repo-specific behavior explicit in the skill description. |

## PROGRESSIVE DISCLOSURE
- `SKILL.md` is the routing surface: frontmatter, trigger description, core workflow, and the smallest useful command set.
- Put bulky material in `references/`: command catalogs, design rubrics, examples, API details, benchmark notes.
- Put deterministic helpers in `scripts/`; document inputs, outputs, and validation next to the script.
- Prefer one skill per invocation mode. If a family has distinct triggers (`ponytail`, `ponytail-review`, `ponytail-audit`), keep separate folders instead of one branching prompt.
- Do not hide external provenance in references. README attribution and `configs/metadata/skills.lock.json` must still identify source URL, license, and commit.

## IMPORT RULES
- Imported external commands are documentation, not instructions to run during import.
- Preserve upstream license as `LICENSE.txt` when a license exists.
- If upstream frontmatter uses unsupported top-level keys, move them under `metadata` and mention that adaptation in README attribution.
- After adding or changing skills, run `stacc sync-metadata --refresh-origin` and `stacc check`.
