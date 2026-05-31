---
name: rust
description: Provides Rust coding best practices for structure, patterns, performance, and error handling. Use when writing or reviewing Rust code, or when the user asks for Rust style guidance.
---

# Rust Best Practices

## Quick Start

Apply these rules by default when touching Rust:

1. Organize code by feature/module, not by file type
2. Keep structs small and focused; split large data into composable types
3. Prefer `Result<T, E>` for recoverable errors, avoid `panic!`
4. Document every `unsafe` block with a `// SAFETY:` rationale
5. Default to `Vec`/`HashMap`; pre-allocate when size is known
6. For CLIs, expose non-interactive flags, stdin/file input, actionable errors, and example-rich help

## Workflow (use this order)

1. Clarify scope: new module, refactor, or review.
2. Organize by feature/module; keep type + impls together.
3. Ensure error handling uses `Result` (avoid `panic!` except invariants).
4. Document `unsafe` blocks with `// SAFETY:` and minimal scope.
5. Choose data structures intentionally and pre-allocate when size is known.
6. For binaries, define the automation contract: flags, stdin, output format, exit codes, and dry-run behavior.
7. Add or update tests (unit tests + rustdoc examples for public APIs; CLI help/error tests for binaries).

## Review Checklist

- Module layout is feature-driven and cohesive.
- `Result` used for recoverable errors; `panic!` justified.
- All `unsafe` blocks have explicit `// SAFETY:` rationale.
- No oversized structs; data is composed cleanly.
- Collections pre-allocated when size is known.
- CLI surfaces can run unattended and explain retry commands on usage errors.
- Tests and doc examples cover edge cases.

## Sub-Skills

Load these for focused guidance on specific topics:

| Skill | When to Use |
|-------|-------------|
| `ownership` | Ownership/borrow errors (E0382, E0597), lifetime design |
| `error-handling` | Result vs Option, anyhow vs thiserror, custom error types |
| `concurrency` | Send/Sync errors, threads vs async, shared state design |
| `zero-cost-abstractions` | Generics vs dyn Trait, object safety, dispatch choice |
| `type-driven-design` | Newtypes, type state, PhantomData, invalid states |
| `performance` | Profiling, allocation reduction, cache, parallelism |
| `agent-friendly-cli` | Rust binaries, clap command trees, diagnostics/codegen tools |
| `anti-patterns` | Code review, clone/unwrap overuse, idiomatic rewrites |
| `coding-guidelines` | Naming, style, modern crate recommendations |

## Local Resources

- `rust.mdc` — authoritative Rust rule file (always applied)

## Code Organization

- **Feature-driven modules**: Keep a struct, its enums, and `impl` blocks together
- **Small, cohesive types**: Split large structs into composable pieces

## Common Patterns

- **Newtype for type safety**: Wrap primitives to avoid ID/value mixups
- **Builder for complex construction**: Use builders for many optional fields
- **Minimal generic bounds**: Put bounds on `impl`/functions, not the type
- **CLI contracts for binaries**: Make command inputs, outputs, exit codes, and destructive behavior explicit

## Performance Guidance

- Default to `Vec` and `HashMap`; switch only with evidence or requirements
- Pre-allocate capacity when the approximate size is known

## Error Handling and Safety

- Use `Result<T, E>` for recoverable errors
- Reserve `panic!` for invariants or unrecoverable bugs
- Add a `// SAFETY:` rationale before every `unsafe` block

## Agent-Friendly CLIs

- Prefer `clap` command trees with real examples in top-level and subcommand help
- Accept stdin (`--stdin` or `--input -`) when callers may pipe generated code
- Provide structured output such as `--json` for automation
- Validate usage before expensive work and return actionable retry examples
- Put destructive writes behind `--dry-run` plus explicit confirmation

## Testing

- Prefer unit tests with `#[test]` in a `tests` module
- Use `rustdoc` examples for public APIs to keep docs executable
- Test CLI help text, usage errors, and structured output for binaries
