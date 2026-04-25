---
name: runtime_adapter_builder
version: 1.0.0
description: Build a new runtime adapter from research — probe, spec, test, register.
triggers: [new adapter, runtime adapter, adapter for]
tags: [adapter, scope:both, runtime:any, risk:high]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [research/SKILL.md, skill_builder/SKILL.md, council_runner/SKILL.md] }
---
# runtime_adapter_builder

Build an `adapters/<runtime_id>/` directory + files when Ciel encounters a new host.

## Operations

- `rab.research(runtime_id)` — deep research; populate capability map.
- `rab.probe(runtime_id)` — capability probe against the live runtime.
- `rab.draft(runtime_id)` — instantiate `templates/adapter.template.md`.
- `rab.test(runtime_id)` — run canned smoke tests.
- `rab.register(runtime_id)` — Council-gated registration.

## I/O Contract

```yaml
io_contract:
  input: { op, runtime_id, "args?" }
  output: { result }
  idempotent: register_only_for_same_version
  side_effects: [fs, network]
```

## Safety

- Registration is high risk (installs a major new capability). Full Council with elevated thresholds.
- Drafted adapter sandbox-tested before Council.
