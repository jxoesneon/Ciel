# improvement.config — Global Self-Improvement

```yaml
# <anchor:start>
improvement:
  global_max_per_day: 20
  local_max_per_day: 10
  trigger_dedup_window_hours: 6
  suppression_days: 7
  sweep_interval: weekly            # daily|weekly|monthly
  checkpoint_stale_hours: 72
  checkpoint_retention_days: 14
  regression:
    watch_invocations: 20
    watch_hours: 48
  auto_tune: true
  auto_tune_range: 0.10            # ±10% is trivial
# <anchor:end>
```

## Notes

- Caps throttle Council runs. Proposals beyond the cap queue.
- `suppression_days` — after a rejected proposal, similar signals are ignored for this period.
- `regression.*` — watch window after a self-modification commit.
- `auto_tune_range` — within ±this fraction, changes to numeric fields are treated as trivial (no Council).

## Triggers

See `self_improvement/TRIGGERS.md` for what fires improvement proposals.
