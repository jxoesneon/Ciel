---
name: ciel-meta-operations
version: 1.0.0
format: skill/1.0
description: CIEL's framework for API standards, UI state auditing, and system configuration.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(audit|configure|call).*(ui|state|api|ciel|ecc)"
    confidence: 0.9
  - pattern: "click path audit"
    confidence: 1.0
---

# CIEL ADAPTATION: Meta Operations (The Orchestrator)

This skill manages the internal logic of CIEL and its interaction with the Claude API and UI state.

## Claude API Standards
1. **Model Selection**: Opus (Architecture/Research), Sonnet (Coding/Logic), Haiku (Classification/High-volume).
2. **Thinking Layer**: Enable `thinking` for complex math or multi-step logic tasks.
3. **Efficiency**: Use Prompt Caching for large system prompts. Use Batches API for non-time-sensitive bulk work (50% cost reduction).

## Click-Path Audit (State Integrity)
- **Problem**: Traditional debugging misses handlers that silently undo each other.
- **Protocol**:
  1. Map State Store actions (`sets` vs `resets`).
  2. Trace EVERY function call in a handler in order.
  3. CHECK: Does a later call undo an earlier state change? (Sequential Undo).
  4. CHECK: Is the final state what the label promises?

## System Configuration (ECC)
- **Installation**: Use the interactive installer to select skill categories (Framework, Database, Workflow).
- **Level**: Choose User-level (`~/.claude/`) for global tools; Project-level (`.claude/`) for repo-specific rules.

## Anti-Patterns
- **Sequential Undo**: `setLoading(true); resetForm();` where reset sets loading to false.
- **Hardcoded API Keys**: Passing `sk-` tokens directly in code instead of env vars.
- **Amnesiac Description**: Writing a skill description that describes "How" instead of "When."
