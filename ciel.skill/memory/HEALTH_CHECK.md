# HEALTH_CHECK — Memory Backend Health Verification

Startup verification + corruption recovery for the active memory backend (MemPalace, Obsidian, or fallbacks).

## Checks

1. **Backend Availability** — Active binary runnable (`mempalace-rs`) OR vault directory accessible & writeable (`.ciel-brain/`).
2. **Schema Version** — `meta/schema_version` matches expected schema (or auto-upgradable).
3. **Partitions Verification** — `ciel-global` and active project partitions present.
4. **RW Self-Test** — Put throw-away key, read back, verify frontmatter/data, delete. Expect no errors.
5. **Path Security Guard** — Verify key and partition sanitization rules are active (`PathTraversalError` check).
6. **Checksum of Recent Entries** — Verify last 10 non-archive entries parseable.

## On Failure

| Failure | Action |
| --- | --- |
| Binary / Vault path missing | Attempt setup / reconnect; on failure → trigger fallback. |
| Path security check failure | Reject operation, log security audit entry, quarantine invalid path. |
| Schema mismatch | Attempt auto-migration; on failure → trigger fallback. |
| Partitions missing | Attempt restore from latest snapshot; on failure → recreate empty + escalate. |
| RW self-test fails | Trigger fallback. Run corruption diagnostic in the background. |
| Checksum failure | Move corrupt entries to `~/.ciel/.attic/corrupt/<ts>/`, reindex, continue. |

## Scheduling

- Startup: always.
- Per-session: once at session start; again on any write error.
- Periodic: configurable (`memory.config.health_check_interval_minutes`, default 60).

## Recovery Without Data Loss

Auto-recovery prefers data preservation. If a choice must be made between availability and integrity, Ciel chooses integrity — entering degraded mode with fallback backend while preserving the original MemPalace partition for forensic inspection.

## Notification

Health failures are activity.log + user-visible summaries (see `observability/`). Silent failure is a Constitutional violation.
