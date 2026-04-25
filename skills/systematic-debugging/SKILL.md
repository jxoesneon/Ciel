---
name: systematic-debugging
version: 1.0.0
format: skill/1.0
description: Ciel's core diagnostic mandate — root cause investigation before any fix.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(bug|error|failure|crash|unexpected|broken).*"
    confidence: 0.9
  - pattern: "debug.*"
    confidence: 1.0
  - pattern: "why does.*fail"
    confidence: 0.95
  - pattern: "fix.*(again|it)"
    confidence: 0.85
---

# CIEL ADAPTATION: Systematic Debugging

This skill enforces CIEL's diagnostic rigor: NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.

## Integration Context

Adapted from `~/.agents/skills/systematic-debugging/`. This is a critical safety and quality mandate for all CIEL-orchestrated troubleshooting.

## The Iron Law

**NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST.**

If you propose a fix without a confirmed root cause: **STOP. Return to Phase 1.**

## The Four Phases

### Phase 1: Root Cause Investigation
- **Evidence Gathering**: Read error messages, traces, and logs completely.
- **Reproduction**: Create a consistent, minimal reproduction of the failure.
- **Trace Back**: Follow the data flow backward from the symptom to the source.

### Phase 2: Pattern Analysis
- **Differential Analysis**: Compare the failing code against similar working examples.
- **Context Mapping**: Identify all dependencies, configurations, and environment variables involved.

### Phase 3: Hypothesis & Minimal Testing
- **Single Variable**: Form a specific hypothesis and test it with the smallest possible change.
- **Verification**: Confirm if the minimal change proves the hypothesis before implementing a full fix.

### Phase 4: Implementation (via TDD)
- **Failing Test**: Create an automated test that reproduces the bug (Red).
- **Root Cause Fix**: Address the source, not the symptom (Green).
- **Architectural Gate**: If 3+ fixes fail, **STOP** and trigger a **Council Review** of the architecture.

## Orchestration Logic

### 1. Triage
The **orchestration** skill must ensure Phase 1 is complete before any sub-task for "fixing" is dispatched.

### 2. High-Risk Diagnostics
For multi-component failures, the **Council of Five** (specifically **Safety** and **Efficiency**) will review the diagnostic plan to ensure proper instrumentation.

### 3. Verification
A bug is "resolved" only when:
- The root cause is identified and addressed.
- A regression test is added to the permanent test suite.
- The **Council of Five** (Coherence/Safety) verifies the fix aligns with system invariants.

---
*Original Documentation preserved at source.*
