---
name: ciel-swarm-orchestration
version: 1.0.0
format: skill/1.0
description: CIEL's framework for multi-agent coordination, task decomposition, and parallel execution.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(swarm|parallel|coordinate|distribute|decompose).*(agents|tasks|workers)"
    confidence: 1.0
  - pattern: "invoke subagents"
    confidence: 1.0
---

# CIEL ADAPTATION: Swarm Orchestration (The Network)

This skill formalizes the "Alloy" swarm model, allowing CIEL to decompose complex tasks into parallel sub-tasks.

## Swarm Protocol
1. **Decompose**: Break the primary task into N independent sub-tasks.
2. **Assign**: Dispatch sub-tasks to specialized sub-agents (Guilds or Personas).
3. **Execute**: Run sub-agents in parallel (if supported) or sequential batches.
4. **Synthesize**: Collect sub-agent outputs and merge them into a single, high-integrity result.
5. **Verify**: The Council of Five performs a coherence check on the synthesized output.

## Coordination Rules
- **Independence**: Sub-tasks MUST not have race conditions on the same file in a single turn.
- **Context Limit**: Minimize the context passed to sub-agents to maximize token efficiency.
- **Reporting**: Every sub-agent MUST report its progress back to the Orchestrator.

## Anti-Patterns
- **The Context Dump**: Passing the entire project history to every sub-agent.
- **The Race Condition**: Modifying the same file from two parallel sub-agents.
- **The Blind Synthesis**: Merging sub-agent outputs without a final coherence check.
