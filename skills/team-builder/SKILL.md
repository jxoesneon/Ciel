---
name: team-builder
version: 1.0.0
format: skill/1.0
description: CIEL's framework for interactive agent-team composition by capability profile. Provides a browsing and selection UX for composing ad-hoc teams from available agent personas.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
side_effects: ["shell"]
triggers:
  - pattern: "(team builder|compose.*team|pick.*agents|browse.*agents|agent picker)"
    confidence: 0.9
  - pattern: "(which agents|select agents|ad-hoc team).*(use|dispatch|work)"
    confidence: 0.8
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Team Builder (Interactive Agent Composition)

This skill provides an interactive agent-picker UX for composing teams by capability profile. While `ciel-swarm-orchestration` automatically decomposes tasks and assigns sub-agents, team-builder is human-in-the-loop: the user browses available agent personas, selects which to deploy, and defines the task interactively. Use this when the user wants deliberate control over team composition rather than automatic dispatch.

## When to Use

- You have multiple agent personas and want to pick which ones to use for a task.
- You want to compose an ad-hoc team from different domains (e.g., Security + SEO + Architecture).
- You want to browse what agents are available before deciding.

## Agent Discovery

Agents are discovered via two methods, merged and deduplicated by name:

1. **`claude agents` command** (primary): Returns all agents known to the CLI — user agents, plugin agents (prefixed `plugin-name:`), and built-in agents.
2. **File glob** (fallback): Reads markdown from `./agents/**/*.md`, `~/.claude/agents/**/*.md`.

Precedence on name collision: user agents > plugin agents > built-in agents. Built-in agents (Explore, Plan) are skipped unless explicitly requested.

## Domain Grouping

- **Subdirectory layout**: Domain inferred from parent folder name (e.g., `engineering/security-engineer.md` → Engineering).
- **Flat layout**: Domain inferred from shared filename prefixes. A prefix qualifies as a domain only if 2+ files share it. Unique-prefix files go to "General". Splits at first `-`, so multi-word domains should use subdirectory layout.

## Selection Flow

1. **Discover**: Run `claude agents`, parse output, read markdown files for names/descriptions.
2. **Present Domain Menu**: Show domains with agent counts; skip empty domains.
3. **Handle Selection**: Accept flexible input — numbers ("1,3"), names ("security + seo"), or "all from engineering". If >5 agents selected, ask user to narrow down (max 5 per team).
4. **Confirm**: Show selected agents, prompt for task description.
5. **Spawn in Parallel**: Read each agent's markdown, spawn all via parallel Agent tool calls with `subagent_type: "general-purpose"`. Each runs independently.
6. **Synthesize**: Collect outputs, present unified report grouped by agent with a synthesis section (agreements, conflicts/tensions, next steps). Skip synthesis if only 1 agent selected.

## Rules

- **Dynamic discovery only**: Never hardcode agent lists — new files auto-appear.
- **Max 5 agents per team**: More than 5 produces diminishing returns and excessive token usage.
- **Parallel dispatch**: All agents run simultaneously via parallel Agent tool calls.
- **Parallel Agent calls, not TeamCreate**: TeamCreate is only needed when agents must debate each other.

## Anti-Patterns

- **Hardcoded Rosters**: Baking agent names into the skill instead of discovering dynamically.
- **Oversized Teams**: Dispatching more than 5 agents — token costs explode, synthesis degrades.
- **No Synthesis**: Presenting raw agent outputs without highlighting agreements and conflicts.
- **Sequential Dispatch**: Running agents one-by-one when they are independent — wastes latency.
