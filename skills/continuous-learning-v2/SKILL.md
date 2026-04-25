---
name: continuous-learning-v2
version: 1.0.0
format: skill/1.0
description: CIEL's primary evolution engine. Uses hook-driven "Instincts" with confidence scoring to learn project-specific and global patterns.
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
triggers:
  - pattern: '(instinct|learning|evolution).*(status|evolve|promote)'
    confidence: 0.95
  - pattern: 'how is (ciel|the agent) evolving'
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Continuous Learning V2 (The Instinct Engine)

This is CIEL's core engine for autonomous evolution. It turns every session into structured knowledge by capturing atomic "instincts"—small, confidence-weighted behaviors learned from direct observation.

## The Instinct Model
An instinct is a discrete, evidence-backed behavior:
- **Trigger**: The specific context that activates the behavior.
- **Action**: The learned "Do X" or "Don't do Y" instruction.
- **Confidence**: 0.3 (Tentative) to 0.9 (Core behavior).
- **Scope**: `project` (scoped to a specific repo) or `global`.

## The Learning Loop
1. **Observe (Hooks)**: Every tool call is captured via Pre/Post-ToolUse hooks, ensuring 100% reliable data collection.
2. **Analyze (Background)**: A background agent (e.g., Haiku) identifies patterns in user corrections, error resolutions, and repetitive workflows.
3. **Persist (MemPalace)**: Learned instincts are stored in the MemPalace Knowledge Graph (`mempalace_kg_add`) or project-specific registries.
4. **Evolve**: Related instincts are clustered and promoted into full Skills, Commands, or Agent Definitions.

## Project Scoping & Promotion
- **Isolation**: React patterns stay in React projects; Python conventions stay in Python projects.
- **Promotion**: When an instinct appears in 2+ projects with high confidence (>= 0.8), it is promoted to the Global Scope.

## Legacy Session Evaluation (Stop-Hook)
CIEL retains the legacy probabilistic session-end analysis from V1 as a lightweight fallback:
1. **Evaluates**: Checks if the session length meets the minimum threshold (default: 10 messages).
2. **Detects**: Identifies recurring patterns in error resolutions, workarounds, and project-specific conventions.
3. **Extracts**: Proposes new skills or updates to existing ones based on the session's "best practices."

## Commands
- `/instinct-status`: Show all learned instincts and their confidence levels.
- `/evolve`: Cluster instincts into new skills or commands.
- `/promote`: Manually move project-scoped instincts to global scope.

## Anti-Patterns
- **Context Pollution**: Applying a project-specific hack globally.
- **Premature Promotion**: Promoting an instinct to global scope based on a single session's data.
- **Ignoring Corrections**: Failing to decay the confidence of an instinct after a user correction.
