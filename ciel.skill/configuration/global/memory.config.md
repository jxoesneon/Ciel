# memory.config — Global Memory

```yaml

# <anchor:start>

memory:
  backend: mempalace           # mempalace|sqlite|filesystem|custom
  auto_update: true
  version_pin: null
  reinstall_check_days: 7
  health_check_interval_minutes: 60
  isolation_strict: true       # Constitutional: locked true
  partition_size_limit_mb: 1024
  fallback_snapshot_retention_days: 30
  custom:
    entry: null
    runtime: null
    endpoint: null
    auth_env: null

# <anchor:end>

```

## Notes

- `backend` — default `mempalace`. Change requires Council (structural).
- `isolation_strict: true` is Constitutional. Cannot be disabled.
- `auto_update` — Ciel upgrades MemPalace-rs to latest compatible on init cadence.
- `version_pin` — freeze to a specific version; Ciel will still warn on security updates.

## Fallback Order

Hard-coded: MemPalace → SQLite → Filesystem → Custom. See `memory/FALLBACK.md`.
