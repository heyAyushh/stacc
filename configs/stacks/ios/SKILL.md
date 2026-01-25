---
name: ios
description: iOS and Swift/SwiftUI development guidance, including UI patterns, Liquid Glass styling, performance audits, view refactors, Swift concurrency, and iOS debugging. Use when building or reviewing iOS apps, SwiftUI views, or troubleshooting iOS issues.
---

# iOS Stack Skill

## Default guidance

Use the focused sub-skills in this folder based on the task type.

## Workflow (use this order)

1. Classify the task (UI pattern, performance, refactor, concurrency, debugging).
2. Load the matching sub-skill(s) and follow their procedure.
3. Make changes with minimal scope; prefer small, testable steps.
4. Validate on simulator/device and check logs.
5. Add or update previews/tests when applicable.

## Review Checklist

- Correct sub-skill chosen and applied.
- SwiftUI view body is simple; state is minimal and predictable.
- Concurrency uses structured tasks; avoids unstructured `Task {}` leaks.
- Performance red flags (layout thrash, heavy body recompute) addressed.
- Debugging notes captured if the issue is intermittent or device-specific.

## Local Resources (sub-skills)

- `swiftui-ui-patterns/SKILL.md` for SwiftUI layout and interaction patterns.
- `swiftui-liquid-glass/SKILL.md` for Liquid Glass visual style and implementation.
- `swiftui-performance-audit/SKILL.md` for performance profiling and optimization checklists.
- `swiftui-view-refactor/SKILL.md` for refactoring and view decomposition workflows.
- `swift-concurrency-expert/SKILL.md` for structured concurrency and async/await guidance.
- `ios-debugger-agent/SKILL.md` for debugging, crash triage, and device/simulator diagnostics.
