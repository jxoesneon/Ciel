---
name: ai-regression-testing
version: 1.0.0
format: skill/1.0
description: Specialized regression testing for AI-assisted development and blind spot mitigation.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "ai-regression.*testing"
    confidence: 0.9
  - pattern: "catch.*blind.*spot"
    confidence: 0.9
  - pattern: "sandbox.*production.*parity"
    confidence: 0.9
  - pattern: "bug-check.*"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: AI Regression Testing

This skill provides CIEL's specialized testing strategy for mitigating systematic blind spots in AI-assisted development. It focuses on contract-first verification and sandbox/production parity.

## Integration Context

Adapted from `~/.agents/skills/ai-regression-testing/`. This skill complements the core **TDD** mandate by targeting common AI failure patterns, such as "hallucinated success" and path inconsistency.

## Core Patterns

### 1. Sandbox/Production Parity
**Problem**: AI fixes the production path but forgets the sandbox/mock path (or vice versa).
**Solution**: Assert identical response shapes in both modes.

### 2. Contract Verification
**Problem**: AI adds a field to the response object but forgets to include it in the underlying data query (e.g., SELECT clause).
**Solution**: Assert all required fields exist in the final API response.

### 3. Error/State Integrity
**Problem**: old data persists in the UI after an error, or optimistic updates lack proper rollback.
**Solution**: Explicit tests for state cleanup and restoration on failure.

## Orchestration Logic

### 1. Post-Fix Verification
After every bug fix (via **systematic-debugging**), CIEL must generate a specific regression test named after the bug (e.g., `BUG-R1 regression`).

### 2. Bug-Check Workflow
Run `npm run test` and `npm run build` **before** any AI-led code review to catch mechanical failures early.

### 3. Strategy: Test Where Bugs Cluster
Focus testing effort on auth, multi-path logic, and state management where AI regressions are most frequent. Do not aim for 100% coverage; aim for 100% regression prevention.

---
*Original Documentation preserved at source.*
