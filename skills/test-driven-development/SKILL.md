---
name: test-driven-development
version: 1.0.0
format: skill/1.0
description: Ciel's core engineering mandate — red-green-refactor for all behavioral changes.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(implement|add|fix).* (feature|bug|capability)"
    confidence: 0.9
  - pattern: "tdd|red-green-refactor"
    confidence: 1.0
  - pattern: "write.*test.*first"
    confidence: 0.95
---

# CIEL ADAPTATION: Test-Driven Development (TDD)

This skill enforces CIEL's "Iron Law" of engineering: no production code without a failing test first.

## Integration Context

Adapted from `~/.agents/skills/test-driven-development/`. This is a foundation-level mandate for all CIEL-orchestrated engineering.

## The Iron Law

**NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST.**

If you wrote code before a test: **Delete it. Start over.**

## The Cycle

1.  **RED**: Write one minimal test for the desired behavior.
2.  **Verify RED**: Run the test and **confirm it fails** for the expected reason.
3.  **GREEN**: Write the **minimal code** required to pass the test.
4.  **Verify GREEN**: Run the test and confirm it passes.
5.  **REFACTOR**: Clean up the code while keeping the test green.

## Orchestration Logic

### 1. Pre-Task Triage
Before any engineering sub-task, the **orchestration** skill must ensure a TDD strategy is defined.

### 2. Council Gate
For high-risk features, the **Council of Five** (specifically **Safety** and **Evolution**) will review the test plan before implementation starts.

### 3. Verification
A task is not "complete" until:
- Every new function/method has a corresponding test.
- All tests pass in the target environment.
- Output is pristine (no warnings/errors).

---
*Original Documentation preserved at source.*
