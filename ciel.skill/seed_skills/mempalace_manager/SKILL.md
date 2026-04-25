---
name: mempalace_manager
version: 1.0.0
description: MemPalace-rs operations — read, write, query, partition management, AAAK.
triggers: [memory, mempalace, recall, remember, store]
tags: [memory, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md], system: [mempalace-rs] }
---
# mempalace_manager

Abstract memory API — backend-neutral wrapper.

## Operations

- `mem.put(partition, key, value, metadata?)`
- `mem.get(partition, key)`
- `mem.query(partition, filter)`
- `mem.search(partition, query, top_k)` — semantic
- `mem.delete(partition, key)`
- `mem.list(partition, prefix)`
- `mem.compact(partition)` — AAAK recompression
- `mem.snapshot(partition, path)`, `mem.restore(partition, path)`
- `mem.with_project(id, fn)` — scoped access
- `mem.lift(key)` — explicit global read from local context (logged)

## I/O Contract

```yaml
io_contract:
  input: { op, partition, args }
  output: { result }
  idempotent: depends
  side_effects: [fs]
```

## Safety

- Partition scoping enforced — cross-scope reads require `with_project()` or `lift()`.
- Constitutional invariant: `isolation_strict` cannot be disabled.
- Selects backend via `memory.config.backend` (mempalace|sqlite|filesystem|custom).
