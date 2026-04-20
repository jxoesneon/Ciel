# acquisition.config — Global Acquisition

```yaml
# <anchor:start>
acquisition:
  tier1_timeout_s: 10
  tier2_timeout_s: 30
  tier3_timeout_s: 120
  total_wall_budget_s: 300
  token_budget: 80000
  tier1_floor: 0.5
  cve_threshold: high
  sandbox_limits:
    cpu_pct: 50
    memory_mb: 512
    wall_s: 60
  sandbox_retention_hours: 48
  min_trust: 0.2
# <anchor:end>
```

## Notes

- `cve_threshold` — skills whose deps include a CVE at or above this severity are auto-rejected pre-Council.
- `sandbox_limits` — hard caps on sandbox resource consumption.
- `min_trust` — sources below this trust score queued for removal.

## Source Management

See `acquisition/SOURCES.md` for the live list of acquisition sources with trust scores.
