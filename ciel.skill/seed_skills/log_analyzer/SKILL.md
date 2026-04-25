---
name: log_analyzer
version: 1.0.0
description: Log parsing, pattern detection, anomaly flagging, structured extraction.
triggers: [log, logs, parse log, anomaly]
tags: [observability, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md] }
---
# log_analyzer

Parse and analyze logs (including Ciel's own `activity.log`).

## Operations

- `logs.parse(path, format?)` — JSONL, plain, common formats.
- `logs.recent(n)`, `logs.by_kind(kind)`, `logs.failures(since)`.
- `logs.search(pattern)`.
- `logs.cost(since)` — aggregate `cost_usd`.
- `logs.anomaly(stream, baseline)` — deviation detection.
- `logs.trace_render(path)` — debug-mode trace renderer.

## I/O Contract

```yaml
io_contract:
  input: { op, args }
  output: { result }
  idempotent: true
  side_effects: [fs]
```

## Integration

Surfaced via `/ciel-status`. Feeds `self_improvement/TRIGGERS.md` for regression and cost signals.
