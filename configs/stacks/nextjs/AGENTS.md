# NEXTJS STACK KNOWLEDGE BASE

## OVERVIEW
`configs/stacks/nextjs/` packages Next.js and React guidance for installation into agent skill/rule directories. It is source content, not the live `docs/` app.

## STRUCTURE
```text
configs/stacks/nextjs/
|-- SKILL.md                         # Stack entrypoint
|-- rules/                           # Cursor MDC rules for Next.js/TypeScript
|-- agentation/                      # Agentation skill package
|-- composition-patterns/            # Component architecture guidance
|-- react-best-practices/            # Large React/Next performance rule corpus
`-- web-interface-guidelines/        # Accessibility and UI interaction guidance
```

## WHERE TO LOOK
| Task | Location | Notes |
| --- | --- | --- |
| Stack entrypoint | `SKILL.md` | Top-level Next.js stack behavior. |
| Cursor rules | `rules/*.mdc` | Installed as stack rules. |
| Composition patterns | `composition-patterns/` | Has its own AGENTS.md; do not repeat details here. |
| React performance | `react-best-practices/` | Has its own AGENTS.md and many rule markdown files. |
| Web UI guidelines | `web-interface-guidelines/` | Has its own AGENTS.md for interaction/accessibility. |
| Agentation | `agentation/SKILL.md` | Separate skill package under the stack. |

## CONVENTIONS
- Keep stack payloads installable. A folder with `SKILL.md`, `metadata.json`, `rules/`, or `AGENTS.md` is package content consumed by `stacc`.
- Child AGENTS files are authoritative inside their leaf package. Update the closest child file when the rule is specific to composition, React runtime behavior, or web-interface details.
- Keep rule filenames descriptive and stable; downstream installs can depend on paths.
- If adding a new sub-bundle, include clear entrypoint metadata (`SKILL.md` and metadata when the sibling package pattern uses it).
- Treat external framework advice as documentation-derived content: add source/cadence/verification notes when scripts refresh it.

## ANTI-PATTERNS
- Do not collapse child packages into one flat Next.js rule file.
- Do not duplicate the long leaf AGENTS content here; this parent file only routes work.
- Do not edit this stack to change the live docs application. Use `/docs/AGENTS.md` and files under `docs/` for that.
- Do not add generated caches, build output, or package installs under this stack.

## COMMANDS
```bash
cargo run -- install --editor codex --scope global --category stack --stack nextjs --dry-run --print-plan
cargo run -- check
```
