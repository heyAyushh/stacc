# docs-frontend-rethink - Work Plan

## TL;DR (For Humans)

**What you'll get:** The STACC docs app keeps its industrial monochrome identity, but the docs pages now behave like a real responsive product: searchable, copyable, theme-aware, readable on mobile, and backed by a maintainable Markdown rendering path.

**Why this approach:** The visual direction was already right; the failures were consistency, responsive data surfaces, copy/search interactions, and build/runtime architecture. The fix kept the brand and replaced brittle implementation details.

**What it will NOT do:** It does not rebrand STACC, add decorative backgrounds, disable mobile zoom, or replace the app with a new UI kit.

**Effort:** Large
**Risk:** Medium - the docs workspace had mixed hoisted and nested dependencies, plus a large dirty worktree from prior UI work.
**Decisions I made for you:** Preserve the STACC.FYI brutalist language, keep App Router server rendering, use Satteri for Markdown HTML rendering, keep client code only for real interactions, and use browser QA as the acceptance gate.

Your next move: review the running site at http://127.0.0.1:4174/ and then request granular commits if the current diff is acceptable.

---

> TL;DR (machine): Large executed docs frontend sweep; Satteri renderer, responsive CSS cleanup, semantic inventory surfaces, copy/search/theme fixes, and browser QA are complete.

## Scope

### Must Have
- Preserve STACC's monochrome editorial/control-surface identity.
- Keep docs in the Next.js workspace, not static HTML clones.
- Render docs Markdown through a maintainable server path.
- Normalize docs shell, route chrome, search, theme, copy, focus, and live feedback.
- Make inventory, skills, and binary/TUI surfaces responsive without nonsensical mobile tables.
- Keep mobile inputs at 16px or larger without disabling user zoom.
- Add real verification: typecheck, build, audit, and browser QA on representative routes.

### Must NOT Have
- No full visual rebrand.
- No decorative gradients, orbs, generic SaaS cards, or random backgrounds.
- No viewport zoom disabling.
- No destructive cleanup of screenshots or unrelated dirty worktree files.
- No operations outside the project root, `.git`, `.env`, or credential files.

## Verification Strategy

> Zero human intervention - all verification was agent-executed.

- Test decision: tests-after via existing Next type/build gates plus browser automation.
- Command evidence:
  - `npm run docs:typecheck` passed.
  - `npm run docs:build` passed and generated 103 static pages.
  - `npm run docs:audit` passed with 0 vulnerabilities.
- Browser evidence: `.omo/evidence/docs-frontend-rethink-browser-qa.md`
- External implementation reference: Next.js `serverExternalPackages` was checked for the Satteri native package bundling boundary.

## Execution Strategy

### Parallel Execution Waves

Wave 1: Rendering and app architecture.
Wave 2: Interaction and responsive UI normalization.
Wave 3: Build/runtime dependency correction.
Wave 4: Browser QA and final polish.

### Dependency Matrix

| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1. Satteri Markdown rendering | Existing docs loader | Build verification, browser route checks | 2, 3 |
| 2. Copy and command interaction normalization | Existing copy panels/search UI | Browser interaction QA | 1, 3 |
| 3. Responsive inventory and docs CSS | Existing CSS/components | Browser overflow QA | 1, 2 |
| 4. Next workspace/build config | Satteri integration | Production build, dev preview | 5 |
| 5. Browser QA and evidence | 1-4 | Final completion | None |

## Todos

- [x] 1. Move docs Markdown rendering to Satteri.
  What to do / Must NOT do: Add `satteri`, remove the old MDX remote compile path from the shell rendering branch, and keep server rendering. Do not turn authored docs into static HTML clones.
  Parallelization: Wave 1 | Blocked by: existing docs loader | Blocks: build verification
  References: `docs/components/docs-shell.tsx`, `docs/package.json`, `package-lock.json`
  Acceptance criteria: `npm run docs:typecheck` and `npm run docs:build` pass.
  QA scenarios: load `/docs/getting-started` and `/docs/architecture` in browser; Markdown body and generated sections render.
  Evidence: `.omo/evidence/docs-frontend-rethink-browser-qa.md`
  Commit: Y | `feat(docs): render markdown with satteri`

- [x] 2. Normalize copy interactions and live feedback.
  What to do / Must NOT do: Make landing install copy and docs code panels report copied/error states. Do not silently swallow clipboard fallback failures.
  Parallelization: Wave 2 | Blocked by: existing copy components | Blocks: browser interaction QA
  References: `docs/components/copy-panel.tsx`, `docs/components/install-copy-card.tsx`, `docs/lib/clipboard.ts`, `docs/components/mdx-components.tsx`
  Acceptance criteria: Browser click sets `data-copy-state="copied"` on landing install card and docs code panel.
  QA scenarios: click quick install card on `/`; click first code panel on `/docs/getting-started`.
  Evidence: `.omo/evidence/docs-frontend-rethink-browser-qa.md`
  Commit: Y | `fix(docs): make copy feedback reliable`

- [x] 3. Replace brittle responsive data surfaces.
  What to do / Must NOT do: Convert installer target rows to a semantic table with mobile card behavior; make dependency and TUI surfaces adaptive; remove stale fake-table selectors. Do not keep huge desktop tables on narrow viewports.
  Parallelization: Wave 2 | Blocked by: existing inventory components | Blocks: responsive browser QA
  References: `docs/components/inventory-panels.tsx`, `docs/app/globals.css`
  Acceptance criteria: `/docs/binary-tui` has no page-level horizontal overflow at 390px and 1280px.
  QA scenarios: browser metrics on `/docs/binary-tui`, desktop and mobile.
  Evidence: `.omo/evidence/docs-frontend-rethink-browser-qa.md`
  Commit: Y | `fix(docs): make inventory surfaces responsive`

- [x] 4. Tighten docs visual consistency and mobile safety.
  What to do / Must NOT do: Normalize tokens, `100dvh`, font loading, search input sizing, code wrapping, theme controls, compact skill card actions, focus-visible, hover/active states, and reduced-motion behavior. Do not hide layout bugs behind viewport zoom disabling.
  Parallelization: Wave 2 | Blocked by: CSS audit | Blocks: browser QA
  References: `docs/app/globals.css`, `docs/app/layout.tsx`, `docs/components/theme-toggle.tsx`, `docs/components/command-search.tsx`
  Acceptance criteria: no input below 16px; no route-level horizontal overflow on tested routes.
  QA scenarios: desktop and mobile browser checks for `/`, `/docs/getting-started`, `/docs/binary-tui`, `/docs/skills`, `/docs/architecture`.
  Evidence: `.omo/evidence/docs-frontend-rethink-browser-qa.md`
  Commit: Y | `style(docs): normalize responsive interaction patterns`

- [x] 5. Correct Next workspace runtime/build config.
  What to do / Must NOT do: Keep production build green with Satteri as a server external package, and keep the dev server usable in the mixed workspace install shape. Do not hand-edit generated dependency folders.
  Parallelization: Wave 3 | Blocked by: Satteri integration | Blocks: final verification
  References: `docs/next.config.mjs`, `docs/package.json`
  Acceptance criteria: `npm run docs:build` passes; `npm run docs:dev` serves http://127.0.0.1:4174/.
  QA scenarios: launch dev server and open live routes in Chrome through Playwright.
  Evidence: `.omo/evidence/docs-frontend-rethink-browser-qa.md`
  Commit: Y | `build(docs): stabilize next workspace resolution`

## Final Verification Wave

- [x] F1. Plan compliance audit
  Result: Implemented scope matches the approved draft defaults: preserve identity, improve structure, normalize interactions, and require browser evidence.

- [x] F2. Code quality review
  Result: Changed files typecheck; no `as any`, `@ts-ignore`, or weakened test gates were introduced.

- [x] F3. Real manual QA
  Result: Playwright drove live pages at desktop and mobile widths, clicked copy controls, and opened command search.

- [x] F4. Scope fidelity
  Result: No rebrand, no destructive cleanup, no work outside project root, and no viewport zoom disabling.

## Commit Strategy

Recommended granular commits after user review:

1. `feat(docs): render markdown with satteri`
   - `docs/components/docs-shell.tsx`
   - `docs/package.json`
   - `package-lock.json`

2. `fix(docs): make copy feedback reliable`
   - `docs/components/copy-panel.tsx`
   - `docs/components/install-copy-card.tsx`
   - `docs/lib/clipboard.ts`
   - `docs/components/mdx-components.tsx`

3. `fix(docs): make inventory surfaces responsive`
   - `docs/components/inventory-panels.tsx`
   - related `docs/app/globals.css` sections

4. `style(docs): normalize responsive chrome`
   - `docs/app/globals.css`
   - `docs/app/layout.tsx`
   - `docs/components/command-search.tsx`
   - `docs/components/theme-toggle.tsx`
   - `docs/components/landing-page.tsx`

5. `build(docs): stabilize next workspace resolution`
   - `docs/next.config.mjs`
   - `docs/package.json`

Do not include unrelated untracked screenshots unless explicitly requested.

## Success Criteria

- [x] The docs app keeps STACC visual identity.
- [x] Markdown docs render through Satteri on the server path.
- [x] The docs app typechecks.
- [x] The docs app production build succeeds.
- [x] The docs app audit reports 0 moderate-or-higher vulnerabilities.
- [x] Browser QA shows no page-level horizontal overflow on tested desktop/mobile routes.
- [x] Copy interactions visibly report copied state.
- [x] Command search opens and uses 16px input text.
- [x] The dev server is running at http://127.0.0.1:4174/.
