# GLOBAL — `~/.ciel/`

See `memory/GLOBAL_STORE.md` for filesystem layout.

## What Lives Here

- All files tracked in `MANIFEST.md` and their integrity records.
- Complete skill registry + installed skills (`skills/`).
- Council records, Council run artifacts.
- Acquisition sources + attempt traces.
- Self-improvement proposals + outcomes.
- Activity log (not git-tracked; significant events mirrored into committed summaries).
- MemPalace global partition data.
- Backups.

## Git Convention

`main` branch, linear history for Ciel-initiated commits. Conventions in `registry/VERSIONING.md`. No force-push; Constitution.

## Mutation Paths

Only through:

- Ciel-initiated routes after appropriate gating.
- User commands (`/ciel-*`).
- External hand-edits (detected by integrity; reconciled or escalated).

## Privacy

Nothing project-specific should leak here. Promoted learnings are generalized first (`acquisition/HARMONIZATION.md` style identifier stripping).

## Backup

`init/BACKUP.md` covers `~/.ciel/` entirely. Users may add a remote.

## Migration Across Machines

Users can migrate Ciel to a new machine by:

1. Backing up `~/.ciel/`.
2. Running `init/scripts/install.sh` on the new machine.
3. Restoring the backup (`ciel backup restore`).
4. Re-running memory health check; MemPalace migration handled automatically if version differs.

Skills, council history, registry, and sources carry over cleanly.
