# SUBAGENTS — Claude Code

`.claude/agents/<name>.md` defines a specialized subagent. Ciel uses subagents for:

- Council of Five members (5 nested agents, all with isolated context)
- Acquisition workers (tier-specific)
- Sandbox runners for candidate skills
- Long-running research tasks

## File Shape

```markdown
---
name: ciel-council-safety
description: Safety lens of Ciel's Council of Five
tools: [Read, Grep, Bash, WebFetch]
---
<body — the persona prompt>
```

## Context Isolation

Each subagent gets a fresh context window. Ciel passes only the relevant bundle (skill metadata, rubric, scoring prompt) rather than the whole orchestrator state. This keeps the parent session's context clean.

## Council Members as Subagents

| Member | Agent file |
| --- | --- |
| Coherence | `.claude/agents/ciel-council-coherence.md` |
| Capability | `.claude/agents/ciel-council-capability.md` |
| Safety | `.claude/agents/ciel-council-safety.md` |
| Efficiency | `.claude/agents/ciel-council-efficiency.md` |
| Evolution | `.claude/agents/ciel-council-evolution.md` |

All installed at init from `council/members/*.md` persona content + `prompts/council/<lens>_stage1.md`.

## Parallel Dispatch

Claude Code supports multiple `Agent` tool calls in a single turn. Ciel's Stage 1 dispatches five `Agent` calls concurrently — one per member — and awaits all completions before entering Stage 2.

## Recursion

Claude Code allows nested subagent invocation. Ciel uses this sparingly (budget pressure). By default, Council members cannot spawn further subagents.

## Installation

See `init/INIT.md`. Each agent file is rendered from a template (`templates/subagent.template.md`) and checksum-tracked.
