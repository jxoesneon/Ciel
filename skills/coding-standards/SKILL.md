---
name: coding-standards
version: 1.0.0
format: skill/1.0
description: Baseline cross-project engineering conventions for readability, immutability, and architecture.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(code|coding).*(standard|convention|quality)"

    confidence: 0.9

  - pattern: "how should I.*(write|structure|format).*(code|function|component)"

    confidence: 0.9

  - pattern: "(review|refactor).*(code|file).*(quality|readability)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Coding-Standards (Engineering Baseline)

This skill provides the baseline engineering conventions applicable across CIEL projects. It establishes the "shared floor" for code quality, focusing on readability, immutability, and architectural soundness.

## Integration Context

Adapted from `~/.agents/skills/coding-standards/`. This skill serves as a foundational reference for the **Auditor** sub-agent during execution and the **Verification-Loop** during finality, ensuring that all code meets CIEL's definition of "high quality."

## Core Principles (The Iron Laws)

1. **Readability First**: Code is read more than written. Prefer descriptive names and self-documenting code over comments.
2. **KISS (Keep It Simple, Stupid)**: Implement the simplest solution that works. Avoid premature optimization and over-engineering.
3. **DRY (Don't Repeat Yourself)**: Extract common logic into reusable functions/components.
4. **YAGNI (You Aren't Gonna Need It)**: Do not build features speculatively.

## Mandated Patterns

### 1. Immutability (CRITICAL)

- **NEVER** mutate variables directly.
- **ALWAYS** use the spread operator (`...`) or array methods that return new instances (e.g., `map`, `filter`).

### 2. Error Handling

- **ALWAYS** use comprehensive `try/catch` blocks for asynchronous operations.
- **NEVER** swallow errors silently. Provide user-friendly error messages and log technical details.

### 3. Code Smells

The **Auditor** sub-agent MUST reject implementations containing:

- Functions exceeding 50 lines (Long Functions).
- Logic nesting deeper than 3 levels (Deep Nesting) — use early returns.
- Unexplained "Magic Numbers" — use named constants.

## Orchestration Logic

- **Implementation Phase**: When a sub-agent generates code, it must consult this skill to ensure naming and structural compliance.
- **Review Phase**: The **Verification-Loop** cross-references the diff against these standards to catch anti-patterns before declaring a task "READY."

---
*Detailed framework-specific patterns (TypeScript, React, API Design) are preserved at the source and referenced as needed.*
