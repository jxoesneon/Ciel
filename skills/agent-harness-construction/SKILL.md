---
name: agent-harness-construction
version: 1.0.0
format: skill/1.0
description: Architecture and optimization guidelines for designing agent tool-sets, action spaces, and recovery paths.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(create|design|build|optimize).*(agent|harness|action space|tool definition)"
    confidence: 0.9
  - pattern: "how should I (structure|format) (tools|observations)"
    confidence: 0.85
---

# CIEL ADAPTATION: Agent Harness Construction (Architecture Design)

This skill provides the architectural principles for designing and optimizing the "harness" through which CIEL sub-agents interact with the world. It ensures that tool definitions are unambiguous and observations are actionable.

## Core Design Principles

### 1. Action Space Granularity
- **Micro-Tools**: For high-risk, irreversible operations (e.g., `git_commit`, `db_migration`). Requires explicit verification.
- **Medium-Tools**: For common iterative loops (e.g., `read_file`, `grep_search`). Optimized for context efficiency.
- **Macro-Tools**: Used ONLY when the latency/cost of round-trips is the dominant bottleneck.

### 2. The Observation Contract
Every tool execution MUST return a structured observation containing:
- `status`: `success` | `warning` | `error`
- `summary`: A concise, one-line explanation of the result.
- `next_actions`: Suggested follow-up steps based on the output.
- `artifacts`: Links or paths to any resources created or modified.

### 3. Error Recovery Paths
Tool definitions MUST include explicit recovery hints in their error messages. Instead of "File not found," return "File not found at /path/x. Did you mean /path/y? Try running `ls` to verify the path."

## Implementation Patterns

- **ReAct Planning**: Use for exploratory tasks with high uncertainty.
- **Hybrid Execution**: Combine ReAct for high-level reasoning with typed, schema-first tool calls for deterministic execution.

## Anti-Patterns
- **Overlapping Semantics**: Creating multiple tools that do essentially the same thing (e.g., `search_code` and `find_text`).
- **Opaque Observations**: Tools that return massive, unformatted blobs of text without success/failure metadata.
- **Context Bloat**: Including entire documentation files in the tool descriptions instead of loading them via the `skill` system.
