---
name: context-budget
version: 1.0.0
format: skill/1.0
description: A proactive audit tool for monitoring and optimizing token overhead across agents, skills, and MCP servers.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(check|audit|view).*(context|token).*(budget|usage|overhead)"

    confidence: 0.9

  - pattern: "/context-budget"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Context Budget (Context Audit)

This skill provides the Orchestrator with a proactive mechanism to monitor and optimize the "Permanent Context" overhead of the system. It identifies bloat in agent definitions, redundant skills, and over-subscribed MCP servers.

## Audit Workflow

### Phase 1: Resource Inventory

The Orchestrator estimates the token footprint of:

- **Agents**: Line counts and description frontmatter.
- **Skills**: Total tokens across `SKILL.md` files.
- **Rules**: Global and project-level rule files.
- **MCP Servers**: Estimated schema overhead (~500 tokens per tool).
- **Project State**: `CLAUDE.md` and `TodoWrite` length.

### Phase 2: Classification & Triage

Components are sorted into three categories:

- **Essential**: Backs an active command or matches project type.
- **On-Demand**: Lazy-loadable or domain-specific.
- **Redundant**: Overlapping logic or rarely used.

### Phase 3: Optimization Recommendations

The Auditor produces a prioritized list of token-saving actions:

- **Agent Thinning**: Reducing verbose descriptions.
- **MCP Pruning**: Removing servers that wrap simple CLI commands.
- **Rule Consolidation**: Merging overlapping language rules.

## The 40% Rule

The Orchestrator should aim to keep the "System Overhead" (permanent context) below 40% of the model's total context window to ensure sufficient headroom for conversation and tool results.

## Anti-Patterns

- **Invisible Bloat**: Keeping 20+ MCP servers active "just in case."
- **Verbose Frontmatter**: Agent descriptions that read like long tutorials.
- **Rule Duplication**: Repeating global rules in project-specific files.
