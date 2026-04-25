---
name: prompt-and-session-engineering
version: 1.0.0
format: skill/1.0
description: CIEL's framework for prompt optimization and session management.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(optimize|rewrite|branch|compact).*(prompt|session|repl)"
    confidence: 0.95
  - pattern: "prompt optimizer"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Prompt & Session Engineering (The Craft Layer)

This skill manages the internal quality of instructions and the health of long-running sessions.

## Prompt Optimization (Advisory)
1. **Diagnosis**: Identify missing context (Tech stack, boundaries, criteria).
2. **Workflow Matching**: Map the intent to CIEL commands (`/plan`, `/tdd`, `/verify`).
3. **Refinement**: Output an optimized prompt that includes explicit scope, tech stack, and verification steps.
4. **Constraint**: Advisory role ONLY. Prohibit executing the task; only provide the optimized text.

## Session Hygiene (REPL)
- **Task Focus**: Keep sessions focused on a single logical unit.
- **Branching**: Branch the session (`/branch`) before performing high-risk or speculative changes.
- **Compaction**: Compact history (`/compact`) after major milestones to preserve context for the final integration.
- **Metrics**: Monitor token usage and budget burn per session.

## Multi-Prompt Splitting (EPIC)
- **Prompt 1**: Research + Plan (Architecture).
- **Prompt 2-N**: Implement one phase per prompt (TDD).
- **Final Prompt**: Integration + Security Review.

## Anti-Patterns
- **Execution in Advice**: Writing code inside a "Prompt Optimization" turn.
- **Amnesiac Resumption**: Resuming a session without loading the previous decisions from `context.json`.
- **The Context Cliff**: Ignoring history growth until the model starts hallucinating or truncating responses.
