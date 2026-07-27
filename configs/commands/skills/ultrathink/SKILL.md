---
name: ultrathink
description: Deep reasoning mode for complex problems requiring maximum cognitive depth. Use when facing difficult architectural decisions, complex debugging, multi-step problem solving, or situations requiring careful analysis before implementation.
---

# Ultrathink - Deep Reasoning Mode

Engage maximum cognitive depth for complex problems.

## Protocol

### 1. Clarify First

Before implementation, ask only questions that materially block a safe or correct decision. Use the host's available user-input mechanism to:
- Confirm understanding of the core problem
- Surface hidden assumptions
- Identify edge cases and constraints
- Validate success criteria

If the request already supplies these details, state the assumptions you will use and proceed.

### 2. Think Deeply

- Break the problem into atomic components
- Consider multiple approaches with trade-offs
- Reason through edge cases and failure modes
- Think about security, performance, and maintainability
- Record a concise decision summary: assumptions, options considered, chosen approach, and verification plan

### 3. Implement Carefully

- Write code incrementally with clear intent
- Use meaningful names that reveal purpose
- Handle errors explicitly at boundaries
- Keep solutions minimal - no speculative features

### 4. Verify Rigorously

After implementation, use available verification tools or an independent reviewer when supported to:
- Confirm the solution addresses the original problem
- Test critical paths and edge cases
- Validate no regressions were introduced
- Ensure code quality standards are met

## Mindset

- Prefer correctness over speed
- Question assumptions ruthlessly
- Make implicit requirements explicit
- Leave code better than you found it
