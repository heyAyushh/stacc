---
name: rust-guidelines
description: Microsoft Pragmatic Rust Guidelines for implementing, reviewing, and documenting idiomatic Rust. Use when working on Rust APIs, error handling, unsafe code, concurrency, async code, performance, testing, documentation, or agent-facing design.
license: MIT
metadata:
  origin_url: https://microsoft.github.io/rust-guidelines/agents/all.txt
  origin_commit: bbf7b03f3a51548f187888fb8c516e8118ebb1c2
---

# Pragmatic Rust Guidelines

Use the bundled Microsoft reference when writing or reviewing Rust code.

## Workflow

1. Identify the relevant concern: API design, correctness, documentation, errors, unsafe code, concurrency, async work, performance, testing, or maintainability.
2. Search [`references/all.txt`](references/all.txt) for the matching guideline title or rule ID (`M-`, `C-`, `S-`, or `L-`).
3. Apply the guideline's recommendation and its rationale to the code under review.
4. When a deviation is intentional, explain the tradeoff and record the relevant rule ID in the review or change notes.

The reference is a generated, consolidated document. Load only the sections relevant to the current Rust task.
