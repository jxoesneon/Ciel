---
name: ciel-quality-and-verification
version: 1.0.0
format: skill/1.0
description: CIEL's framework for evidence-first verification, TDD, and write-time quality enforcement.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(verify|check|test|lint).*(completion|evidence|tdd|terminal)"
    confidence: 0.95
  - pattern: "verification before completion"
    confidence: 1.0
---

# CIEL ADAPTATION: Quality & Verification (The Integrity Layer)

This skill formalizes the "Evidence before Claims" mandate. it prioritizes fresh verification and disciplined implementation loops.

## The Iron Law (Hard Gate)
**NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE.**
If you haven't run the verification command in the CURRENT message, you cannot claim it passes.

## TDD implementation loop
1. **RED**: Write a failing test that reproduces the bug or exercises the new feature.
2. **GREEN**: Apply the minimal implementation required to pass the test.
3. **REFACTOR**: Improve code quality while maintaining GREEN state.
4. **Coverage**: Mandate **80% line coverage** for all production logic.

## Write-Time Enforcement (Plankton)
- **Silent Phase**: Run formatters (Ruff/Biome) on every file edit.
- **Delegation Phase**: Route complex violations to specific model tiers (Haiku for style, Sonnet for logic, Opus for types).
- **Enforcement**: Block legacy package managers (pip/npm) in favor of modern runtimes (uv/bun).

## Terminal Ops (Evidence-First)
- **Workflow**: surface -> Evidence -> Action -> Status.
- **Integrity**: Distinguish between "changed locally" and "verified locally."
- **Verification**: Name the specific command or test that proves the fix.

## Anti-Patterns
- **The "Should" Trap**: Claiming code is correct because it "should work" or "looks right."
- **Amnesiac Regression**: Passing a test once and then moving on without verifying the RED-GREEN cycle.
- **Linter-Passing ≠ Compiler-Passing**: Claiming success because the linter is clean while the build is broken.
