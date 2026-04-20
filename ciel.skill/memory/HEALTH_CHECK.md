# HEALTH_CHECK — MemPalace-rs

Startup verification + corruption recovery for the memory backend.

## Checks

1. **Binary present & runnable** — `mempalace-rs --version` succeeds.
2. **Schema version** — `get ciel-global meta/schema_version` matches expected (or upgradable).
3. **Partitions listed** — `ciel-global` and current project's partition both present.
4. **RW self-test** — put a throw-away key, read back, delete. Expect no errors.
5. **Checksum of recent entries** — verify last 10 non-archive entries parseable.

## On Failure

| Failure | Action |
| --- | --- |
| Binary missing | Attempt `INSTALL.md`; on failure → fallback. |
| Schema mismatch | Attempt auto-migration; on failure → fallback. |
| Partitions missing | Attempt restore from latest backup; on failure → recreate empty + escalate. |
| RW self-test fails | Fallback. Run corruption diagnostic in the background. |
| Checksum failure | Move corrupt entries to `~/.ciel/.attic/corrupt/<ts>/`, reindex, continue. |

## Scheduling

- Startup: always.
- Per-session: once at session start; again on any write error.
- Periodic: configurable (`memory.config.health_check_interval_minutes`, default 60).

## Recovery Without Data Loss

Auto-recovery prefers data preservation. If a choice must be made between availability and integrity, Ciel chooses integrity — entering degraded mode with fallback backend while preserving the original MemPalace partition for forensic inspection.

## Notification

Health failures are activity.log + user-visible summaries (see `observability/`). Silent failure is a Constitutional violation.
