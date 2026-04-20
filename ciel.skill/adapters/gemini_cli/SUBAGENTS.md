# SUBAGENTS — Gemini CLI

Gemini CLI subagents live under `.gemini/agents/<name>.md` and can be invoked:

- implicitly via natural delegation,
- explicitly via `@<agent-name>` syntax,
- programmatically via the parallel dispatch primitive.

## Recursion Guard

Subagents **cannot** spawn further subagents. This is enforced at the runtime level and is not configurable. It is a load-bearing constraint for Ciel's Council design (see `COUNCIL_INVOCATION.md`).

## Parallel Dispatch

Native. Multiple subagents can be dispatched in a single turn, each with its own isolated context window. Results return as structured summaries that the parent session can reason over.

## File Shape

```markdown
---
name: ciel-council-safety
description: Safety lens for Ciel's Council of Five
tools: [read_file, grep, shell, web_fetch]
model: gemini-3-flash
---
<persona + rubric body>
```

Note that the `model` field lets Ciel pick a cheaper model for lightweight Council lenses (Coherence, Efficiency) and a stronger one for Safety. This is a Ciel optimization not available under Claude Code.

## Forced Delegation (`@syntax`)

Ciel uses `@` syntax when she needs a *specific* subagent and must guarantee it runs:

```text
@ciel-council-safety please score <artifact>
```

## A2A (remote) Subagents

For very long tasks, subagents may be remote (`agent_card_url`). See `A2A.md`. Ciel treats remote subagents as normal subagents with higher latency and stricter logging.

## Installation

All Ciel-managed subagents are rendered from `templates/subagent.template.md` with lens-specific content. Checksum-tracked.
