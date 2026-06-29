---
slug: docs-frontend-rethink
status: executed
intent: approved-by-user
pending-action: none
approach: preserve STACC identity, restructure docs UI patterns, normalize interactions, and require browser evidence before execution handoff
---

# Draft: docs-frontend-rethink

## Components (topology ledger)
<!-- Lock the SHAPE before depth. One row per top-level component that can succeed or fail independently. -->
<!-- id | outcome (one line) | status: active|deferred | evidence path -->
1. Identity and design-system contract | Keep industrial monochrome STACC identity while allowing CSS/component restructuring | active | DESIGN.md:3-91, docs/app/globals.css
2. Docs shell and route chrome | Make header/nav/search/theme/footer consistent across docs and skill detail routes | active | docs/components/docs-shell.tsx:43-184, docs/app/docs/skills/[skillSlug]/page.tsx
3. Content and MDX architecture | Keep docs content human-maintainable and server-first; centralize MDX/component mapping | active | docs/lib/docs.ts:1-120, docs/components/mdx-components.tsx
4. Inventory, skills, and binary/TUI surfaces | Replace brittle visual grids with semantic adaptive components that preserve meaning on mobile | active | docs/components/inventory-panels.tsx, docs/app/globals.css
5. Interaction system | Normalize search, theme, copy, external links, focus-visible, reduced motion, and live feedback | active | docs/components/command-search.tsx, docs/components/theme-toggle.tsx, docs/components/copy-panel.tsx, docs/lib/clipboard.ts
6. Verification and evidence | Add agent-executable browser QA and build/type gates for landing and docs routes | active | docs/package.json:8-13, docs/AGENTS.md

## Open assumptions (announced defaults)
<!-- Intent is UNCLEAR: research resolves ambiguity, defaults are adopted (not asked), and each is surfaced in the plan's human TL;DR for veto. -->
<!-- assumption | adopted default | rationale | reversible? -->
1. Visual direction | Preserve the current STACC brutalist/editorial identity, not a full redesign | DESIGN.md already defines the intended identity; the issue is inconsistent execution, not brand direction | reversible
2. CSS architecture | Tokens in DESIGN.md and CSS custom properties stay the source of truth; no Tailwind/theme migration unless a later execution step proves it reduces complexity | Existing app already uses a large global CSS surface and Tailwind v4; minimizing infrastructure churn reduces risk | reversible
3. Content architecture | Keep server-first App Router content; improve the current Markdown/MDX contract instead of forcing a full `@next/mdx` migration immediately | Current docs parse frontmatter manually and compile MDX in the shell; Next official MDX is an option, but migration is not required to fix UX | reversible
4. Tables and ledgers | Use semantic HTML first: native tables for real tabular data, lists/cards for item collections, and horizontal scroll only where the table itself is essential | MDN/WAI guidance favors preserving table semantics; WCAG reflow requires avoiding two-dimensional scrolling except where essential | reversible
5. Client boundaries | Keep client code limited to real browser interactions: command search, theme, copy, dialogs, and canvas | React/Next official guidance favors server components by default and low client boundaries | reversible
6. Mobile inputs | Prevent iOS input zoom with 16px minimum form-control text, not viewport zoom disabling | Preserves accessibility and user zoom | reversible
7. QA level | Require browser evidence for `/`, `/docs/getting-started`, `/docs/architecture`, `/docs/installation`, `/docs/skills`, one skill detail route, `/docs/binary-tui`, and `/docs/configurations` at 320, 375, 768, 1024, and desktop widths | Current failures are visual/responsive; build success alone is insufficient | reversible
8. Worktree handling | Treat current dirty docs changes and screenshot artifacts as pre-existing; execution must not revert unrelated changes | `git status --short` shows many modified/untracked docs files | reversible

## Findings (cited - path:lines)
- Root docs topology is centralized but mixed: `DocsShell` compiles MDX, builds nav, loads search inventory, renders sidebars, and renders footer in one component (`docs/components/docs-shell.tsx:43-184`).
- Theme logic has two sources: inline bootstrap script in `docs/app/layout.tsx:22-37` and client state/apply logic in `docs/components/theme-toggle.tsx:11-107`.
- Docs content discovery is flat Markdown with manual YAML frontmatter parsing (`docs/lib/docs.ts:25-120`), while MDX components provide copyable code/launch panels (`docs/components/mdx-components.tsx`).
- `DESIGN.md` is present and specific: monochrome industrial documentation, compressed display type, mono operational copy, borders-first depth, and responsive legibility (`DESIGN.md:3-91`).
- Current CSS still has many local one-off layout rules, raw colors, small font sizes, `minmax(0, 1fr)` tracks, `overflow-wrap:anywhere`, and shadows that do not consistently map to the design contract (`docs/app/globals.css`, grep evidence recorded in session).
- Current worktree is dirty across docs routes/components/libs/content plus untracked screenshots and new files; plan execution must be staged and reviewable, not a broad rewrite.
- No local server responded on `127.0.0.1:4174` or `127.0.0.1:3000` during planning; current visual claims must be reverified by execution.
- External defaults used: Next MDX/App Router docs, React Server Components and `use client`, MDN table/semantic/accessibility docs, W3C WCAG target size/focus/reflow, web.dev responsive guidance, and documentation style examples.

## Decisions (with rationale)
- Plan as UNCLEAR: user asked to rethink hard and take inspiration, so planning adopts defaults instead of asking a design interview.
- Preserve identity but not structure: keep STACC's monochrome poster/control-surface language while allowing component extraction, CSS sectioning, and responsive rewrites.
- Make responsive behavior content-driven: breakpoints should follow readable words, usable controls, and semantic data shape, not fixed device labels.
- Normalize interactions before polishing visuals: search, copy, theme, focus, and live feedback are trust-critical docs behaviors.
- Make browser QA a first-class task with evidence artifacts; absence of horizontal scrollbars is not enough because `overflow-x: clip` can hide bugs.

## Scope IN
- Write a decision-complete implementation plan for docs frontend structure, CSS/token cleanup, interaction normalization, adaptive data surfaces, content/MDX architecture, and verification.
- Include exact file references, route matrix, acceptance criteria, happy/failure QA, and commit strategy.
- Preserve STACC.FYI branding, current Anton/IBM Plex Mono identity, monochrome palette, hard borders, and docs/landing distinction.
- Cover landing, docs shell, architecture, getting-started, installation, skills list/detail, binary/TUI, configurations, search, theme, copy, and dependency/source links.

## Scope OUT (Must NOT have)
- No full visual rebrand, no decorative gradients/orbs, no generic SaaS card redesign.
- No product code edits before approval.
- No destructive cleanup of untracked screenshots/artifacts without explicit permission.
- No operations outside the project root, no `.git`, `.env`, or credential files.
- No viewport zoom disabling to solve mobile input zoom.
- No migration to a new UI framework or component kit unless the plan proves a narrow need.

## Open questions
None for approval. The defaults above are reversible and should be vetoed at the approval gate if wrong.

## Approval gate
status: executed
pending action: none
approval: user said "do that" after being told the implementation slice was complete but the formal `.omo` plan was not.
brief: plan artifact completed, execution reconciled against the shipped docs frontend changes, and browser/build evidence recorded in `.omo/evidence/docs-frontend-rethink-browser-qa.md`.
