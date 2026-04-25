---
name: strategic-compact
version: 1.0.0
format: skill/1.0
description: A phase-aware context management strategy that suggests manual compaction at logical task boundaries.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(when|should I) (compact|clear context)"
    confidence: 0.9
  - pattern: "/compact"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Strategic Compact (Phase-Based Memory)

This skill formalizes the practice of "Strategic Compaction"—manual invocation of the `/compact` command at logical boundaries rather than relying on arbitrary, token-triggered auto-compaction.

## The Compaction Decision Guide

| Boundary | Compact? | Rationale |
| :--- | :--- | :--- |
| **Research → Planning** | **YES** | Research traces are bulky; the Plan is the distilled output. |
| **Planning → Implementation** | **YES** | The plan is in `TodoWrite`; clear the reasoning space for code. |
| **Implementation → Testing** | **MAYBE** | Keep if tests require deep code context; compact if focus shifts. |
| **Debug → Next Task** | **YES** | Error traces and logs heavily pollute unrelated work. |
| **Mid-Implementation** | **NO** | Losing variable names and partial state is catastrophic. |

## Operational Protocol
1. **Pre-Compact Audit**: Before compacting, ensure all vital state (Plans, Findings, Decisions) is written to durable storage (`MEMPALACE`, `TodoWrite`, or project files).
2. **Summary Command**: Use `/compact [summary]` to preserve a high-level intent marker across the compaction boundary.
3. **Trigger Threshold**: Suggest compaction to the user every 50 tool calls if a major phase transition has occurred.

## Benefits
- **Response Coherence**: Prevents the model from becoming "confused" by stale reasoning from a prior task.
- **Efficiency**: Maximizes available context for active implementation.
- **Determinism**: Ensures that the most important context (The Plan) is always in the most recent (high-priority) part of the window.

## Anti-Patterns
- **Blind Compaction**: Compacting in the middle of a complex multi-file edit.
- **The Amnesia Trap**: Compacting before writing down the current implementation hypothesis.
