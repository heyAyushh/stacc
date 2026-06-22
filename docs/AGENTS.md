# DOCS KNOWLEDGE BASE

## OVERVIEW
`docs/` is the Next.js workspace for the STACC landing page and Markdown-backed documentation. It is managed by the root npm workspace and should stay separate from the Rust installer logic.

## STRUCTURE
```text
docs/
|-- app/                  # App Router pages, layout, global CSS
|-- components/           # Landing, docs shell, MDX widgets, canvas animation
|-- content/              # Source Markdown pages with YAML frontmatter
|-- lib/docs.ts           # Markdown file discovery and frontmatter validation
|-- package.json          # Next/React/Tailwind workspace manifest
`-- next.config.mjs
```

## WHERE TO LOOK
| Task | Location | Notes |
| --- | --- | --- |
| Landing route | `app/page.tsx`, `components/landing-page.tsx` | STACC Variant-inspired homepage. |
| Docs route | `app/docs/[slug]/page.tsx`, `components/docs-shell.tsx` | Static Markdown docs pages. |
| Markdown content | `content/*.md` | Frontmatter drives title, nav order, sections. |
| Markdown loader | `lib/docs.ts` | Validates frontmatter and sorts pages by `order`. |
| Generated fills | `components/docs-page-fill.tsx` | Slug-keyed sections, catalogs, code panels, and live inventory. |
| Wave animation | `components/wave-canvas.tsx` | Client component; verify in browser, not build only. |
| Styling | `app/globals.css` | Tailwind v4 plus scoped landing/docs CSS. |

## CONVENTIONS
- Add docs pages as `.md` files in `content/` with `title`, `eyebrow`, `description`, `order`, and `sections`.
- Keep `/docs` redirecting to the first actual docs slug unless navigation changes.
- Keep authored docs as plain Markdown. Put generated code panels, catalogs, and inventory blocks in `components/docs-page-fill.tsx`.
- Keep docs navigation derived from `getAllDocsPages()`; do not duplicate page links by hand in `DocsShell`.
- Treat the landing page and docs shell as production UI. Check responsive layout and canvas rendering in a browser.
- Use root scripts (`npm run docs:*`) from the repo root unless package-local execution is required.

## ANTI-PATTERNS
- Do not add standalone HTML/CSS pages as the source of truth for docs.
- Do not hardcode docs slugs in multiple places when `content/*.md` can drive them.
- Do not rely on a green `next build` alone for animation or responsive behavior; inspect the rendered route.
- Do not put Rust installer behavior or generated config metadata in this workspace.

## COMMANDS
```bash
npm run docs:dev
npm run docs:typecheck
npm run docs:build
npm run docs:audit
```

## MANUAL QA
- Open `http://127.0.0.1:4174/` for the landing page when the dev server is running.
- Open `/docs/getting-started`, `/docs/architecture`, and `/docs/installation` after Markdown or generated-fill changes.
- Verify active nav, right-side section links, code panel text, no page-level horizontal overflow, and visible wave canvas motion.
