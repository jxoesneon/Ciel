# ADAPTER — Xcode Coding Assistant

Dedicated capability adapter for Codex running inside Xcode.

## Capability Flags

```yaml
runtime: xcode
floor: { skills: true, subagents: false, mcp: true, shell: true, fs: true, context: true }
enhanced:
  hooks: false                 # Xcode MCP exposes build/test/diagnostic actions, not lifecycle hooks
  parallel_subagents: false    # no native nested agent primitive exposed in Xcode
  plan_mode: true              # host supports explicit plan/checklist updates
  permissions: true            # via managed filesystem sandbox and command approvals
  prompt_cache: false          # not exposed
  otel: false                  # not exposed
  model_switch: false          # not exposed to the adapter
  computer_use: false
  xcode_tools: true            # native project, build, test, preview, diagnostics, and docs tools
```

## File Layout Expected

- `.ciel/project.json` — project-local Ciel metadata
- `AGENTS.md` / project instruction files — persistent project context when present
- Xcode project navigator — preferred project-aware file discovery and editing surface
- `~/.ciel/` — global Ciel state, registry, memory, and activity log

## Installation Footprint

At init (see `init/INIT.md`), Ciel:

1. Ensures `<project>/.ciel/project.json` records `runtime: "xcode"`.
2. Records Xcode context files and project conventions in the local `.ciel/` domain.
3. Uses Xcode MCP tools as the preferred route for project structure, diagnostics, builds, tests, previews, and Apple documentation lookup.

## Route Map

| Ciel route | Xcode mechanism |
| --- | --- |
| Skill activation | Host loads `SKILL.md`; adapter reads additional skill files on demand |
| Subagent (nested) | Inline persona in-context (serial only) |
| Parallel dispatch | Sequential fallback unless host exposes multi-agent tools separately |
| Pre-flight gate | Managed sandbox, approval prompts, and inline risk checks |
| Post-execution scoring | Inline after tool calls; append to Ciel activity log when available |
| MCP | Native MCP tool calls, especially `xcode-tools` |
| Context injection | Project instruction files and `.ciel/project.json` |
| Diagnostics | `XcodeRefreshCodeIssuesInFile` for focused compiler feedback |
| Build | `BuildProject` through Xcode's build system |
| Tests | `GetTestList`, `RunSomeTests`, and `RunAllTests` |
| Documentation | `DocumentationSearch` for Apple framework APIs |
| UI preview | `RenderPreview` for SwiftUI previews |
| Shell isolation | Managed filesystem sandbox plus explicit escalation |
| Headless script | Shell execution when needed, scoped to project root |

## Project Interaction Strategy

Prefer Xcode MCP tools over generic shell commands when working with Xcode projects:

1. Use `XcodeLS`, `XcodeGlob`, `XcodeGrep`, and `XcodeRead` for project-aware discovery.
2. Use `XcodeUpdate` and `XcodeWrite` for project-aware edits.
3. Use `XcodeRefreshCodeIssuesInFile` for fast validation after Swift edits.
4. Use `BuildProject` when a full compile is needed.
5. Use `DocumentationSearch` for current Apple framework behavior.

## Council Invocation Strategy

Xcode does not expose a native nested-agent primitive through this adapter. Council of Five runs as an **inline sequential deliberation**:

1. Load each councilor persona in order.
2. Record risk, evidence, and recommendation from each persona.
3. Synthesize a single decision before applying high-risk changes.

See `COUNCIL_INVOCATION.md`.

## Hook-Equivalent Setup

Xcode has no exposed persistent hook runtime. This adapter uses inline preflight and postflight gates for every blockable route. See `HOOKS.md`, `MCP.md`, and `CONTEXT_FILES.md`.
