# Template — new runtime adapter

Fills out `adapters/<runtime_id>/` when `runtime_adapter_builder/SKILL.md` instantiates.

## Directory

```text
adapters/{{runtime_id}}/
├── ADAPTER.md
├── HOOKS.md                   # if runtime exposes hooks
├── SUBAGENTS.md               # delegation primitive
├── MCP.md                     # if MCP supported
├── CONTEXT_FILES.md           # runtime context file handling
├── COUNCIL_INVOCATION.md      # Council topology on this runtime
└── <runtime-specific>.md      # e.g. PLAN_MODE, ULTRAPLAN, A2A, etc.
```

## Capability Map Template

```yaml
runtime: {{id}}
floor:
  skills: true
  subagents: {{true|false}}
  mcp: {{true|false}}
  shell: true
  fs: true
  context: true
enhanced:
  hooks: {{true|false}}
  parallel_subagents: {{true|false}}
  plan_mode: {{true|false}}
  permissions: {{true|false}}
  computer_use: {{false|preview|true}}
  ...runtime-specific...
```

## Route Map

| Route | Mechanism |
| --- | --- |
| Skill activation | {{how}} |
| Subagent | {{how}} |
| Pre-flight | {{how}} |
| Post-execution | {{how}} |
| MCP | {{how}} |
| Context | {{how}} |
| Headless | {{how}} |

## Council Topology

{{nested|parallel-top-level|sequential}}

## Installation Footprint

{{files written at init}}
