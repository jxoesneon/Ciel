---
name: ciel-internal-ops
version: 1.0.0
format: skill/1.0
description: Meta-operations for CIEL configuration, LLM cost management, and parallel workflows.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(configure|manage|split).*(ciel|cost|dmux|parallel|ecc)"
    confidence: 1.0
  - pattern: "configure-ecc"
    confidence: 1.0
---

# CIEL ADAPTATION: Internal Operations (The Meta-Framework)

This skill formalizes the self-management of CIEL, from installation to cost-aware execution.

## CIEL Configuration (ECC)
1. **Install Wizards**: Use for selective skill/rule deployment. Choose between Core and Niche packs.
2. **Verification**: After any CIEL update, verify file existence and scan for broken path references.
3. **Tailoring**: Adjust coverage targets and patterns to the project's specific tech stack.

## Cost-Aware LLM Pipeline
- **Routing**: Sonnet (Complex/Batch) -> Haiku (Simple/Atomic). Threshold: 10k chars or 30 items.
- **Caching**: Mandate Prompt Caching for system prompts > 1024 tokens.
- **Tracking**: Use immutable trackers to maintain a running spend-log. Fail fast on budget overrun.

## Parallel Workflows (dmux)
- **Isolation**: Use `dmux` or `DevFleet` to split independent tasks (e.g., Research vs Implement).
- **Git Worktrees**: ALWAYS create separate worktrees for parallel agents to prevent merge collisions.
- **Seeding**: Use `seedPaths` to copy local context (scripts, draft plans) into worker worktrees.

## Anti-Patterns
- **Token Drain**: Running 10+ parallel agents on a single task. Limit to 3-5 workers.
- **Global Mutation**: Letting parallel agents edit the same shared state or root configuration.
- **Budget Blindness**: Processing 100k items through Opus without a cost-routing check.
