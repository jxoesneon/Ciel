# BACKUP — `~/.ciel/` Snapshots

Periodic snapshots complementary to git history.

## Why Both

- **Git** captures diffs and history; excellent for rollbacks of tracked files.
- **Backup** captures untracked state (MemPalace partition, sqlite db, fallback kv, checkpoints).

## Schedule

```yaml
backup:
  cadence: daily       # none|hourly|daily|weekly
  retention_count: 14
  retention_days: 30
  target: ~/.ciel/backups/
```

## What's Included

- `~/.ciel/` full tree **except** `backups/`, `archive/`, `sandbox/`, `.cache/`.
- MemPalace partition snapshots via `mempalace-rs snapshot <partition> <path>`.
- SQLite fallback db copied with WAL checkpoint.

## Format

tar.zst per snapshot:

```text
~/.ciel/backups/ciel-backup-20260115T093000Z.tar.zst
```

## Integrity

Each snapshot has a sidecar `.sha256`. On restore, hash is verified before extraction.

## Restore

`ciel backup restore <path>` or `/ciel-backup restore <path>`:

1. Stop any in-flight operation.
2. Move current `~/.ciel/` aside to `~/.ciel.pre-restore-<ts>/`.
3. Extract snapshot.
4. Run `INTEGRITY.md`.
5. Run `memory/HEALTH_CHECK.md`.
6. On success, commit a `backup_restore:` marker to git so the event is traceable.
7. On failure, restore the moved-aside directory and escalate.

## Remote

Not configured by default. Users may configure a remote backup destination (S3, rsync target, etc.) via `configuration/global/ciel.config.md` `backup.remote`.

## Pruning

Oldest snapshots beyond `retention_count` or `retention_days` (whichever is stricter) are pruned by the scheduled sweep.
