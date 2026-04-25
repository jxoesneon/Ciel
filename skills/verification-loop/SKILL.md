---
name: verification-loop
version: 1.0.0
format: skill/1.0
description: Comprehensive E2E validation and quality gate enforcement.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "verify.*(changes|implementation|all)"
    confidence: 1.0
  - pattern: "run.*(quality-gate|validation-loop)"
    confidence: 0.95
  - pattern: "is.*(ready|complete).*(for|to|PR)"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Verification-Loop (The Finality Layer)

This skill serves as CIEL's "Finality Layer," providing the definitive evidence required to close a task. It ensures that every implementation is not just "functional," but "high-quality" according to CIEL's engineering standards.

## Integration Context

Adapted from `~/.agents/skills/verification-loop/`. This skill is the terminal state of the **do** (Execution) process. It provides the structured reporting needed for the **Council of Five** to issue a final verdict on a task's success.

## Orchestration Logic

### 1. The Evidence Chain
Verification must start by reading the **make-plan**'s verification checklists and the **brainstorming** spec's success criteria.

### 2. Multi-Stage Validation
CIEL must execute the following phases, reporting results for each:
- **Build & Types**: Ensure zero regressions in compilation or type safety.
- **Lint & Style**: Enforce **ai-first-engineering** and **coding-standards**.
- **Test Suite**: Run unit, integration, and E2E tests with coverage reporting.
- **Security Scan**: Check for leaked secrets, debug logs, and injection vectors.
- **Diff Audit**: A final manual-style review of all changed files.

### 3. The Verification Report
Output a standardized CIEL Verification Report (see template below). This report is the primary artifact for the final **Council** review.

### 4. Memory Finalization
Upon a successful "READY" report, CIEL must:
- Update the Knowledge Graph in **MemPalace** to mark the `ImplementedFeature` as `Verified`.
- Archive the task-specific specs and plans.

### 5. Transition
The terminal state of this skill is a final report to the user, optionally followed by a `finishing-a-development-branch` call.

---
*Original Verification Phases and Output Format preserved at source.*
