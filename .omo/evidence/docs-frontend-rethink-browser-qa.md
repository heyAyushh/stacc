# docs-frontend-rethink Browser QA Evidence

Generated: 2026-06-29T20:13:04.610Z

## DoneClaim

PASS: yes

## Exact Commands

```bash
curl -i --max-time 5 http://127.0.0.1:4174/
timeout 180s node .omo/evidence/docs-frontend-rethink-browser-live/qa-cdp-broad.mjs > .omo/evidence/docs-frontend-rethink-browser-live/browser-qa-broad-stdout.log 2> .omo/evidence/docs-frontend-rethink-browser-live/browser-qa-broad-stderr.log; printf '%s\n' "$?" > .omo/evidence/docs-frontend-rethink-browser-live/browser-qa-broad-shell-exit.txt
timeout 180s npm run docs:audit > .omo/evidence/docs-frontend-rethink-browser-live/docs-audit.log 2>&1; printf '%s\n' "$?" > .omo/evidence/docs-frontend-rethink-browser-live/docs-audit-exit.txt
find . -path './.git' -prune -o -path './docs/node_modules' -prune -o -path './docs/.next' -prune -o \( -name 'Default' -o -name 'Profile 1' -o -name 'Guest Profile' -o -name 'Cookies' -o -name 'History' -o -name 'Login Data' -o -name 'Local State' \) -print | sort
git status --short -- . ':!.git' > .omo/evidence/docs-frontend-rethink-browser-live/git-status-after-broad.txt
```

## Browser Matrix

Tool: Chrome/147.0.7727.102 through Chrome CDP.
Base URL: http://127.0.0.1:4174
Server probe: HTTP 200

Routes:
- /
- /docs/getting-started
- /docs/architecture
- /docs/installation
- /docs/skills
- /docs/binary-tui
- /docs/configurations
- /docs/skills/codex-skills-babysit-pr

Viewports:
- 320x844
- 375x844
- 768x900
- 1024x900
- 1280x900

Route/viewport checks: 40 PASS, 0 FAIL.

Assertions per route/viewport: HTTP load succeeded, no console/page errors, no root horizontal overflow, and no visible input/editable below 16px.

## Interaction Checks

- interaction:landing-copy: PASS; artifacts: interaction-landing-copy-broad
- interaction:docs-copy: PASS; artifacts: interaction-docs-copy-broad
- interaction:command-search: PASS; artifacts: interaction-command-search-broad

## Cleanup Receipt

Chrome temp user-data-dir: /var/folders/j6/4dpy4_z5197d8xfwpbrkd1tc0000gn/T/stacc-docs-qa-chrome-jCoahj
Cleanup proof: `.omo/evidence/docs-frontend-rethink-browser-live/cleanup-receipt-broad.txt` reports `userDataDirRemoved=true` and `repoProfileArtifactScanCount=0`.

## Audit Verification

`timeout 180s npm run docs:audit` exited 0 and reported `found 0 vulnerabilities`.

## Artifacts

- `.omo/evidence/docs-frontend-rethink-browser-live/browser-qa-broad-results.json`
- `.omo/evidence/docs-frontend-rethink-browser-live/docs-audit.log`
- `.omo/evidence/docs-frontend-rethink-browser-live/docs-audit-exit.txt`
- `.omo/evidence/docs-frontend-rethink-browser-live/cleanup-receipt-broad.txt`
- `.omo/evidence/docs-frontend-rethink-browser-live/git-status-after-broad.txt`
- `.omo/evidence/docs-frontend-rethink-browser-live/shot-*.png` for every route/viewport
- `.omo/evidence/docs-frontend-rethink-browser-live/interaction-*-broad.png` for interaction checks

## Manual QA Matrix

The full `manualQa` matrix is embedded in `browser-qa-broad-results.json` with `surfaceEvidence`, `adversarialCases`, and `artifactRefs`.

## Risks

- Existing dirty worktree product changes predate this QA run; this pass only wrote evidence files.
- Chrome emitted host-level updater/crashpad noise in its own log; page-level console and runtime error assertions were clean.
