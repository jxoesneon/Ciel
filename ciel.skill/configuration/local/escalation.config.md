# escalation.config — Local Escalation Threshold

```yaml
# <anchor:start>
escalation:
  # Auto-detected value set by init/CALIBRATION.md.
  # Do not edit `auto_detected` directly; use `override` to set a value.
  auto_detected: null          # research | development | production | regulated
  override: null               # same values; takes precedence when set
  override_reason: null
  override_set_by: null
  override_set_at: null
# <anchor:end>
```

## Notes

- `effective = override ?? auto_detected`.
- Constitutional floor: `override` cannot be more permissive than `auto_detected` by more than one category step, and cannot go below `research`.
- Override shifts the risk gate mapping (`risk/ESCALATION_LADDER.md`).

## Categories

- `research` — most permissive; only critical escalates.
- `development` — default; mid/high Council-gated.
- `production` — conservative; high+ always escalates.
- `regulated` — most restrictive; all mid+ escalates.

## Setting

- `/ciel override set <category>`
- Or edit this file; Ciel reloads on next op.
