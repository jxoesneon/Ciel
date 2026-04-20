# overrides.config — Local Overrides

Explicit overrides of global config values for this project.

```yaml
# <anchor:start>
overrides:
  # Any leaf field from configuration/global/*.config.md may be overridden here.
  # Constitutional invariants cannot be overridden.
  # Example:
  # router:
  #   fast_path_floor: 0.85
  # risk:
  #   high_threshold: 5.0
  # observability:
  #   log_verbosity: debug
# <anchor:end>
```

## Notes

- Leaves only; structural changes (backend swap) must go through global with Council gating.
- Ciel warns when an override crosses a Constitutional floor and refuses to apply it.
- An override in this file shadows the global default just for this project; other projects are unaffected.

## Managing Overrides

- `/ciel config override set <key> <value>` — adds or updates.
- `/ciel config override unset <key>` — removes.
- `/ciel config show effective` — shows resolved effective config for this project.

## Hand-Edits

Safe. Ciel reloads on next operation after detecting file change.
