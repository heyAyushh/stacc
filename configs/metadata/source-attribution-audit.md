# Source Attribution Audit

Audit date: 2026-07-27

This table records the current upstream evidence for every canonical stacc skill package discovered by `stacc sync-metadata`. It separates exact copied packages, historical imports, broad subtree imports, reference-derived local packages, and stacc-authored local skills so freshness checks can be interpreted at the right granularity. Tracked non-canonical mirrors are inventoried separately below.

Method:
- Read all local `SKILL.md` packages included by `stacc sync-metadata`.
- Added `configs/commands/skills/` to metadata discovery so command-as-skill packages are included.
- Queried GitHub repository trees with `gh api repos/<owner>/<repo>/git/trees/HEAD?recursive=1`.
- Queried the historical `mattpocock/skills` `v1` tree and compared Git blob identities for the four legacy packages.
- Spot-compared content for ambiguous Cursor, Expo split-out, Matt Pocock, Emil Kowalski, and Vercel paths.

Status:
- `exact-current`: the local package has a current exact upstream path.
- `subtree-current`: the local package is part of a broader imported upstream subtree.
- `reference-derived`: the local package was made from an upstream reference file, not a complete upstream skill folder.
- `local-wrapper`: stacc-authored skill wrapper around external rule or config content.
- `historical-exact`: the local package exactly matches a package on a pinned historical upstream branch.
- `historical-adapted`: the local package is adapted from a package on a pinned historical upstream branch.
- `payload-mismatch`: a current upstream path exists, but the local payload does not match it.
- `local-original`: stacc-authored, no external freshness target.

Freshness:
- `yes` means the declared imported commit differs from the current upstream repository HEAD.
- `no` means the declared imported commit matches the current upstream repository HEAD.
- `not pinned` means an external source exists but no declared commit is recorded.
- `n/a` means there is no external source target.

## All Skill Package Mappings

| Local path | Source mapping | Status | License | Repo-head stale | Notes |
| --- | --- | --- | --- | --- | --- |
| `configs/plugins/codex/skills/babysit-pr/` | `openai/codex/.codex/skills/babysit-pr` | `exact-current` | `Apache-2.0` | `yes` | Current upstream path found; see source mapping. |
| `configs/commands/skills/clean-gone/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/commit/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/commit-push/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/commit-push-pr/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/council/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/create-command/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/deslop/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/explore/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/fix-pr/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/init/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/iterate-browser/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/onboard-new-developer/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/rebase/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/refactor/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/simplify/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/ultrathink/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/commands/skills/visualize/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc command-as-skill wrapper generated from the tracked command library. |
| `configs/stacks/engineering/cli-for-agents/` | `cursor/plugins/cli-for-agent/skills/cli-for-agents` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/plugins/cursor/skills/continual-learning/` | `cursor/plugins/continual-learning/skills/continual-learning` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/productivity/create-learning-path/` | `cursor/plugins/teaching/skills/create-learning-path` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/review/deslop/` | `cursor/plugins/cursor-team-kit/skills/deslop` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/plugins/cursor/skills/orchestrate/` | `cursor/plugins/orchestrate/skills/orchestrate` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/productivity/run-learning-retrospective/` | `cursor/plugins/teaching/skills/run-learning-retrospective` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/review/thermo-nuclear-code-quality-review/` | `cursor/plugins/cursor-team-kit/skills/thermo-nuclear-code-quality-review` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/productivity/what-did-i-get-done/` | `cursor/plugins/cursor-team-kit/skills/what-did-i-get-done` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/ios/add-app-clip/` | `expo/skills/plugins/expo/skills/add-app-clip` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/skills/agent-browser/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/design/apple-design/` | `emilkowalski/skills/skills/apple-design` | `exact-current` | `MIT` | `yes` | Imported at `56de6f5`; upstream path and repository MIT license verified. |
| `configs/stacks/ios/audio-math-haptics/` | `heyAyushh/audio-math-haptics/skill/audio-math-haptics` | `exact-current` | `MIT` | `no` | Current upstream path found; see source mapping. |
| `configs/stacks/engineering/bash-expert/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/design/brandkit/` | `Leonxlnx/taste-skill/skills/brandkit` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/design/brutalist-skill/` | `Leonxlnx/taste-skill/skills/brutalist-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/building-native-ui/` | `expo/skills/plugins/expo/skills/building-native-ui` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/productivity/caveman/` | `mattpocock/skills@v1/caveman` | `historical-exact` | `MIT` | `yes` | Local `SKILL.md` and `LICENSE.txt` Git blobs exactly match the pinned `v1` branch at `8a54bc3`; repository default HEAD is newer. |
| `configs/stacks/engineering/changelog-generator/` | `ComposioHQ/awesome-claude-skills/changelog-generator` | `exact-current` | `Apache-2.0` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/skills/diagnose/` | `mattpocock/skills/skills/engineering/diagnosing-bugs` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/eas-update-insights/` | `expo/skills/plugins/expo/skills/eas-update-insights` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/design/emil-design-eng/` | `emilkowalski/skills/skills/emil-design-eng` | `exact-current` | `MIT` | `yes` | Imported at `ecf66bb`; the upstream repository moved from `emilkowalski/skill` to `emilkowalski/skills` and now publishes an MIT license. |
| `configs/stacks/expo/expo-api-routes/` | `expo/skills/plugins/expo/skills/expo-api-routes` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/expo-brownfield/` | `expo/skills/plugins/expo/skills/expo-brownfield` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/expo-cicd-workflows/` | `expo/skills/plugins/expo/skills/expo-cicd-workflows` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/expo-deployment/` | `expo/skills/plugins/expo/skills/expo-deployment` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/expo-dev-client/` | `expo/skills/plugins/expo/skills/expo-dev-client` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/expo-module/` | `expo/skills/plugins/expo/skills/expo-module` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/expo-tailwind-setup/` | `expo/skills/plugins/expo/skills/expo-tailwind-setup` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/expo-ui-jetpack-compose/` | `expo/skills/plugins/expo/skills/expo-ui/references/jetpack-compose.md` | `reference-derived` | `MIT` | `yes` | Local split-out skill derived from an exact upstream reference file, not an upstream skill folder. |
| `configs/stacks/expo/expo-ui-swift-ui/` | `expo/skills/plugins/expo/skills/expo-ui/references/swift-ui.md` | `reference-derived` | `MIT` | `yes` | Local split-out skill derived from an exact upstream reference file, not an upstream skill folder. |
| `configs/skills/find-skills/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/design/frontend-design/` | `anthropics/skills/skills/frontend-design` | `exact-current` | `Apache-2.0` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/design/gpt-tasteskill/` | `Leonxlnx/taste-skill/skills/gpt-tasteskill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/skills/gui-automation/` | `trycua/cua/skills/gui-automation` | `exact-current` | `MIT` | `yes` | Imported at `73fe822`; current upstream path and MIT license found. |
| `configs/stacks/productivity/grill-me/` | `mattpocock/skills@v1/grill-me` | `historical-exact` | `MIT` | `yes` | Local `SKILL.md` Git blob exactly matches the pinned `v1` branch at `8a54bc3`; repository default HEAD is newer. |
| `configs/stacks/engineering/grill-with-docs/` | `mattpocock/skills/skills/engineering/grill-with-docs` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/design/hallmark/` | `nutlope/hallmark/skills/hallmark` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/skills/handoff/` | `mattpocock/skills/skills/productivity/handoff` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/design/image-to-code-skill/` | `Leonxlnx/taste-skill/skills/image-to-code-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/design/imagegen-frontend-mobile/` | `Leonxlnx/taste-skill/skills/imagegen-frontend-mobile` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/design/imagegen-frontend-web/` | `Leonxlnx/taste-skill/skills/imagegen-frontend-web` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/engineering/improve-codebase-architecture/` | `mattpocock/skills/skills/engineering/improve-codebase-architecture` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/review/karpathy-guidelines/` | `forrestchang/andrej-karpathy-skills/skills/karpathy-guidelines` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/engineering/mcp-builder/` | `anthropics/skills/skills/mcp-builder` | `exact-current` | `Apache-2.0` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/design/minimalist-skill/` | `Leonxlnx/taste-skill/skills/minimalist-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/native-data-fetching/` | `expo/skills/plugins/expo/skills/native-data-fetching` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/productivity/output-skill/` | `Leonxlnx/taste-skill/skills/output-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/review/ponytail/` | `DietrichGebert/ponytail/skills/ponytail` | `exact-current` | `MIT` | `yes` | Imported from the upstream skills subtree at `40e50d9`. |
| `configs/stacks/review/ponytail-audit/` | `DietrichGebert/ponytail/skills/ponytail-audit` | `exact-current` | `MIT` | `yes` | Imported from the upstream skills subtree at `40e50d9`. |
| `configs/stacks/review/ponytail-debt/` | `DietrichGebert/ponytail/skills/ponytail-debt` | `exact-current` | `MIT` | `yes` | Imported from the upstream skills subtree at `40e50d9`. |
| `configs/stacks/review/ponytail-gain/` | `DietrichGebert/ponytail/skills/ponytail-gain` | `exact-current` | `MIT` | `yes` | Imported from the upstream skills subtree at `40e50d9`. |
| `configs/stacks/review/ponytail-help/` | `DietrichGebert/ponytail/skills/ponytail-help` | `exact-current` | `MIT` | `yes` | Imported from the upstream skills subtree at `40e50d9`. |
| `configs/stacks/review/ponytail-review/` | `DietrichGebert/ponytail/skills/ponytail-review` | `exact-current` | `MIT` | `yes` | Imported from the upstream skills subtree at `40e50d9`. |
| `configs/stacks/engineering/prototype/` | `mattpocock/skills/skills/engineering/prototype` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/review/react-doctor/` | `millionco/react-doctor/skills/react-doctor` | `exact-current` | `LicenseRef-Million-Modified-MIT` | `yes` | Imported at `0b64af5`; current upstream path and modified-MIT license file found. |
| `configs/stacks/design/redesign-skill/` | `Leonxlnx/taste-skill/skills/redesign-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/skills/skill-creator/` | `anthropics/skills/skills/skill-creator` | `exact-current` | `Apache-2.0` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/design/soft-skill/` | `Leonxlnx/taste-skill/skills/soft-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/design/stitch-skill/` | `Leonxlnx/taste-skill/skills/stitch-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/design/taste-skill/` | `Leonxlnx/taste-skill/skills/taste-skill` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/design/taste-skill-v1/` | `Leonxlnx/taste-skill/skills/taste-skill-v1` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/skills/tdd/` | `mattpocock/skills/skills/engineering/tdd` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/engineering/to-issues/` | `mattpocock/skills/skills/engineering/to-issues` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/engineering/to-prd/` | `mattpocock/skills/skills/engineering/to-prd` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/engineering/triage/` | `mattpocock/skills/skills/engineering/triage` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/skills/ultragoal/` | `jxnl/dots/agents/skills/ultragoal` | `exact-current` | `LicenseRef-No-Published-License` | `yes` | Imported at `1eb180f`; GitHub reports no detected license and the repository tree publishes no license file. |
| `configs/stacks/expo/upgrading-expo/` | `expo/skills/plugins/expo/skills/upgrading-expo` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/stacks/expo/use-dom/` | `expo/skills/plugins/expo/skills/use-dom` | `exact-current` | `MIT` | `yes` | Current upstream path found; see source mapping. |
| `configs/skills/using-git-worktrees/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/engineering/write-a-skill/` | `mattpocock/skills@v1/write-a-skill` | `historical-exact` | `MIT` | `yes` | Local `SKILL.md` Git blob exactly matches the pinned `v1` branch at `8a54bc3`; repository default HEAD is newer. |
| `configs/skills/writing-great-skills/` | `mattpocock/skills/skills/productivity/writing-great-skills` | `exact-current` | `MIT` | `yes` | Imported at `d574778`; current upstream path and MIT license found. |
| `configs/stacks/productivity/zoom-out/` | `mattpocock/skills@v1/zoom-out` | `historical-adapted` | `MIT` | `yes` | Adapted from the pinned `v1` package at `8a54bc3`; local frontmatter differs and repository default HEAD is newer. |
| `configs/stacks/bun/` | `local stacc` | `local-original` | `MIT AND CC0-1.0` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/databases/` | `local stacc` | `local-original` | `MIT AND CC0-1.0` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/design/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored progressive router for bundled design skills. |
| `configs/stacks/engineering/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored progressive router for bundled engineering workflows. |
| `configs/stacks/expo/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored progressive router for bundled Expo skills. |
| `configs/stacks/ios/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/ios/ios-debugger-agent/` | `Dimillian/Skills/ios-debugger-agent` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/ios/swift-concurrency-expert/` | `Dimillian/Skills/swift-concurrency-expert` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/ios/swiftui-liquid-glass/` | `Dimillian/Skills/swiftui-liquid-glass` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/ios/swiftui-performance-audit/` | `Dimillian/Skills/swiftui-performance-audit` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/ios/swiftui-ui-patterns/` | `Dimillian/Skills/swiftui-ui-patterns` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/ios/swiftui-view-refactor/` | `Dimillian/Skills/swiftui-view-refactor` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/nextjs/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/nextjs/agentation/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/nextjs/composition-patterns/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/nextjs/react-best-practices/` | `vercel-labs/agent-skills/skills/react-best-practices` | `exact-current` | `MIT` | `no` | Imported from Vercel's agent skills at `7c180d9`; package frontmatter declares MIT. |
| `configs/stacks/nextjs/web-interface-guidelines/` | `vercel-labs/web-interface-guidelines` | `exact-current` | `MIT` | `no` | Imported from the standalone Vercel Labs package at `4e799d4`; upstream repository publishes an MIT license. |
| `configs/stacks/productivity/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored progressive router for bundled productivity modes. |
| `configs/stacks/react-native/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/review/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored progressive router for bundled review modes. |
| `configs/stacks/rust/` | `actionbook/rust-skills/skills` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/agent-friendly-cli/` | `actionbook/rust-skills/skills/domain-cli plus local cli-for-agents adaptation` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/anti-patterns/` | `actionbook/rust-skills/skills/m15-anti-pattern` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/coding-guidelines/` | `actionbook/rust-skills/skills/coding-guidelines` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/concurrency/` | `actionbook/rust-skills/skills/m07-concurrency` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/error-handling/` | `actionbook/rust-skills/skills/m06-error-handling` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/ownership/` | `actionbook/rust-skills/skills/m01-ownership` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/performance/` | `actionbook/rust-skills/skills/m10-performance` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/type-driven-design/` | `actionbook/rust-skills/skills/m05-type-driven` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/rust/zero-cost-abstractions/` | `actionbook/rust-skills/skills/m04-zero-cost` | `subtree-current` | `MIT` | `not pinned` | Part of the imported Rust skills subtree; local names are normalized. |
| `configs/stacks/solana/` | `solana-foundation/solana-dev-skill/skill` | `exact-current` | `MIT` | `not pinned` | Current upstream path found; see source mapping. |
| `configs/stacks/turborepo/` | `local stacc` | `local-original` | `MIT` | `n/a` | Stacc-authored skill package with no external freshness target. |
| `configs/stacks/typescript/` | `sanjeed5/awesome-cursor-rules-mdc/rules-mdc/typescript.mdc` | `local-wrapper` | `CC0-1.0` | `not pinned` | Stacc skill wrapper around external Cursor rule-pack content. |
| `configs/stacks/typescript/typescript/` | `sanjeed5/awesome-cursor-rules-mdc/rules-mdc/typescript.mdc` | `local-wrapper` | `CC0-1.0` | `not pinned` | Stacc skill wrapper around external Cursor rule-pack content. |

## Tracked Non-Canonical Skill Mirrors

These packages are tracked in editor-specific folders but are outside the canonical `configs/` discovery roots, so they do not appear in `skills.lock.json`.

| Local path | Source mapping | Status | License | Repo-head stale | Notes |
| --- | --- | --- | --- | --- | --- |
| `.agents/skills/review-animations/` | `emilkowalski/skills/skills/review-animations` | `payload-mismatch` | `MIT` | `not pinned` | Project-local adaptation with an explicit upstream URL and vendored MIT license; not part of the installer catalog. |
| `.agents/skills/setup-matt-pocock-skills/`, `.claude/skills/setup-matt-pocock-skills/`, `.codex/skills/setup-matt-pocock-skills/`, `.cursor/skills*/setup-matt-pocock-skills/`, `.opencode/skills*/setup-matt-pocock-skills/` | `mattpocock/skills/skills/engineering/setup-matt-pocock-skills` | `exact-current` | `MIT` | `not pinned` | Tracked installed and backup mirrors with vendored MIT license files; no canonical `configs/` package currently exists. |

## Non-Skill References

| Local path | Status | Notes |
| --- | --- | --- |
| `configs/plugins/codex/plugins.json` | reference-only | Optional marketplace entry for LazyCodex; no LazyCodex payload is vendored. |
| `configs/plugins/cursor/agents/agents-memory-updater.md` | exact-current | `cursor/plugins/continual-learning/agents/agents-memory-updater.md`; same upstream plugin as the continual-learning hook. |
| `configs/plugins/cursor/hooks/continual-learning/` | exact-current | `cursor/plugins/continual-learning/hooks`; hook package, not a skill package. |
| `configs/commands/ultrathink.md`, `configs/commands/init.md`, `configs/commands/review-pr.md`, `configs/agents/verifier.md` | local-original | stacc-authored non-skill references. |
