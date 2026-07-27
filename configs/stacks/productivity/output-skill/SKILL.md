---
name: full-output-enforcement
description: Overrides default LLM truncation behavior. Enforces complete code generation, bans placeholder patterns, and handles token-limit splits cleanly. Apply to any task requiring exhaustive, unabridged output.
license: MIT
---

# Full-Output Enforcement

## Activation and baseline

Apply this mode only when the user explicitly asks for exhaustive, unabridged, or full output. Within that request's stated scope, a partial output is broken: if the user asks for a full file, deliver the full file; if they ask for 5 components, deliver 5 components. Do not use this mode to expand a request for a concise answer.

## Banned Output Patterns

The following are hard failures when they silently replace requested content:

**In code blocks:** `// ...`, `// rest of code`, `// implement here`, `// TODO`, `/* ... */`, `// similar to above`, `// continue pattern`, `// add more as needed`, or bare `...` standing in for omitted code.

Literal TODOs and ellipses are allowed when the user requests them, they appear faithfully in supplied source, or they are real code or data rather than omitted content.

**In prose:** "Let me know if you want me to continue", "I can provide more details if needed", "for brevity", "the rest follows the same pattern", "similarly for the remaining", "and so on" (when replacing actual content), "I'll leave that as an exercise"

**Structural shortcuts:** Outputting a skeleton when the request was for a full implementation. Showing the first and last section while skipping the middle. Replacing repeated logic with one example and a description. Describing what code should do instead of writing it.

## Execution Process

1. **Scope** — Read the full request. Count how many distinct deliverables are expected (files, functions, sections, answers). Lock that number.
2. **Build** — Generate every requested deliverable completely. No partial drafts or "you can extend this later" in place of requested content.
3. **Cross-check** — Before output, re-read the original request. Compare your deliverable count against the scope count. If anything is missing, add it before responding.

## Handling Long Outputs

When a response approaches the token limit:

- Do not compress remaining sections to squeeze them in.
- Do not skip ahead to a conclusion.
- Write at full quality up to a clean breakpoint (end of a function, end of a file, end of a section).
- End with:

```
[PAUSED — X of Y complete. Send "continue" to resume from: next section name]
```

On "continue", pick up exactly where you stopped. No recap, no repetition.

## Quick Check

Before finalizing any response, verify:
- No banned pattern silently replaces requested content
- Every item requested within this mode's scope is present and finished
- Code blocks contain actual runnable code, not descriptions of what code would do
- Nothing requested in full was shortened to save space
