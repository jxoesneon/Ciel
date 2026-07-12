---
title: Ciel — Runtime Adapters
project_note: subsystem
type: project-note
tags: ["project-note","subsystem"]
status: active
created: 2026-07-11
updated: 2026-07-11
---

# Ciel — Runtime Adapters

Ciel supports multiple runtime adapters so the same skill package can operate across different AI agent environments.

## Supported runtimes

| Runtime | Adapter path | Status |
|---------|--------------|--------|
| Claude Code | `ciel.skill/adapters/claude_code/` | full |
| Gemini CLI | `ciel.skill/adapters/gemini_cli/` | full |
| Windsurf | `ciel.skill/adapters/windsurf/` | full |
| Generic | `ciel.skill/adapters/generic/` | probe-and-adapt |

## Adapter responsibilities

- Detect runtime capabilities and constraints.
- Install runtime-specific hooks (e.g., `.claude/hooks/` vs `.gemini/hooks/`).
- Map Ciel's abstract operations to runtime-native commands.
- Handle checkpointing and session persistence per runtime.

## Devin integration

- Devin CLI hooks are documented as "Coming Soon".
- Until available, Ciel uses explicit `mempalace` MCP calls for cross-session persistence.
- The `mempalace` MCP server remains available alongside the Obsidian brain as a working-memory tool.

## Capability probe

- `ciel.skill/adapters/generic/CAPABILITY_PROBE.md` describes how the generic adapter introspects an unknown runtime and advertises capabilities back to the router.

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
