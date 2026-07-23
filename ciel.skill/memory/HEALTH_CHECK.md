# HEALTH_CHECK — Obsidian Backend

Startup verification + corruption recovery for the Obsidian memory backend.

## Checks

1. **Obsidian running & Local REST API reachable** — GET `/` returns `status: OK`.
2. **API key valid** — authenticated read/write round-trip succeeds.
3. **Vault path present** — `OBSIDIAN_VAULT_PATH` points to a directory containing `obsidian-brain`.
4. **Hybrid search reachable** — `obsidian-hybrid-search` health endpoint responds.
5. **RW self-test** — put a throw-away key, read back, delete. Expect no errors.
6. **Checksum of recent entries** — verify last 10 non-archive entries parseable.

## On Failure

| Failure | Action |
| --- | --- |
| Obsidian not running | Prompt user to start Obsidian; retry once, then fallback. |
| API key invalid / missing | Direct user to Local REST API settings to regenerate/copy key. |
| Vault path missing | Prompt user to open `obsidian-brain/` as a vault. |
| Hybrid search down | Attempt to start (`npx obsidian-hybrid-search serve`); on failure degrade to vault listing. |
| RW self-test fails | Fallback. Run corruption diagnostic in the background. |
| Checksum failure | Move corrupt entries to `~/.ciel/.attic/corrupt/<ts>/`, reindex, continue. |

## Scheduling

- Startup: always.
- Per-session: once at session start; again on any write error.
- Periodic: configurable (`memory.config.health_check_interval_minutes`, default 60).

## Recovery Without Data Loss

Auto-recovery prefers data preservation. If a choice must be made between availability and integrity, Ciel chooses integrity — entering degraded mode with fallback backend while preserving the original Obsidian vault for forensic inspection.

## Notification

Health failures are activity.log + user-visible summaries (see `observability/`). Silent failure is a Constitutional violation.
