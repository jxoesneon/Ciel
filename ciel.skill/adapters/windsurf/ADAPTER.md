# ADAPTER — Windsurf

Full capability adapter for Windsurf (Codeium's agent IDE).

## Capability Flags

```yaml
runtime: windsurf
floor: { skills: true, subagents: false, mcp: true, shell: true, fs: true, context: true }
enhanced:
  hooks: true                  # Native via ~/.codeium/windsurf/hooks.json
  parallel_subagents: false    # Cascade is single-threaded
  plan_mode: false             # Not native (uses permission patterns)
  permissions: true            # via rules file
  prompt_cache: false          # Not exposed
  otel: false                  # Not exposed
  model_switch: true           # model selection in UI
  computer_use: false
  ultraplan: false
  remote_control: false
  workflows: true              # Native via .windsurf/workflows/
```

## File Layout Expected

- `.windsurf/rules` — project-level context and rules (equivalent to CLAUDE.md)
- `.windsurf/skills/` — installed skills (including ciel.skill unpacked)
- `.windsurf/memory` — persistent memory for Cascade
- `~/.windsurf/` — global configuration

## Installation Footprint

At init (see `init/INIT.md`), Ciel:

1. Unpacks `ciel.skill` to `.windsurf/skills/ciel/`
2. Injects a compact identity block into `.windsurf/rules` (or creates it)
3. Sets up core context files for Cascade to reference

## Route Map

| Ciel route | Windsurf mechanism |
| --- | --- |
| Skill activation | Reference via context injection into `.windsurf/rules` |
| Subagent (nested) | Inline persona in-context (serial only) |
| Parallel dispatch | Sequential fallback (no parallel agents) |
| Pre-flight gate | Inline check before tool calls |
| Post-execution scoring | Inline after tool calls |
| MCP | Native MCP server support via settings |
| Context injection | `.windsurf/rules` hierarchy |
| Long task | Manual session management |
| UI automation | Not available |
| Shell isolation | Permission prompt patterns |
| Headless script | Not available |

## Context Files Strategy

Windsurf uses `.windsurf/rules` for persistent project instructions. Ciel injects:

1. Core identity (compact)
2. Trigger phrases for Ciel invocation
3. Capability boundaries

See `CONTEXT_FILES.md` for format.

## Council Invocation Strategy

Without nested subagents, Council runs as an **inline deliberation**:

1. Load all five councilor personas into context
2. Run sequential rounds (not parallel)
3. Chairman synthesizes and decides

See `COUNCIL_INVOCATION.md`.

## Native Hooks Integration

Windsurf Cascade supports 12 hook events via `~/.codeium/windsurf/hooks.json`. Ciel maps its safety and observability layer to these native hooks:

| Ciel System | Windsurf Hook | Purpose |
| --- | --- | --- |
| Safety pre-flight | `pre_write_code` | Block protected file modifications |
| Safety pre-flight | `pre_run_command` | Block dangerous commands |
| Safety pre-flight | `pre_mcp_tool_use` | Vet MCP tool invocations |
| Activity logging | `post_cascade_response` | Log full trajectory to `~/.ciel/activity.log` |
| Git tracking | `post_write_code` | Auto-stage changes for Ciel commits |

See `HOOKS.md` for full configuration.

## Named Checkpoints Integration

Windsurf Cascade provides **Named Checkpoints and Reverts** — native point-in-time snapshots of project state. Ciel integrates checkpoints for:

- **Pre-flight safety**: Recommend checkpoints before high-risk operations
- **Council deliberation**: Pre-approval snapshots for reversible decisions
- **Self-improvement recovery**: Rollback points for experimental changes
- **Audit trail**: Named checkpoints linked to Council decisions

See `CHECKPOINTS.md` for checkpoint conventions and hook integration.

## Spaces Integration

Windsurf **Spaces** (Agent Command Center) groups agent sessions, PRs, files, and context for a task/project. Ciel leverages Spaces for:

- **Space-level Council governance**: Council context shared across all Space sessions
- **Multi-agent workflows**: Parallel Council deliberation across sessions
- **Context inheritance**: New sessions inherit Space-level Ciel configuration
- **Project-centric organization**: Two-domain model (global + local) extended to Spaces

See `SPACES.md` for Space governance patterns and Ciel integration.

## Workflows Integration

Windsurf supports `.windsurf/workflows/*.md` invoked via `/workflow-name`. Ciel provides:

- `init/workflows/windsurf/ciel-council.md` — Council deliberation workflow
- `init/workflows/windsurf/ciel-acquire.md` — Skill acquisition workflow
- `init/workflows/windsurf/ciel-improve.md` — Self-improvement workflow

## Degraded Features

- **No parallel subagents**: Council and multi-agent tasks run sequentially
- **No plan mode**: Uses "describe before execute" prompt pattern
