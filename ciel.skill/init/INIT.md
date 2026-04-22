# INIT — Master Initialization Ceremony

Run on first load, on `/ciel-init`, and on any session that detects an integrity mismatch.

## Phases

1. **Cold-start check** — if `~/.ciel/` absent, invoke `BOOTSTRAP.md` + `scripts/setup.py`.
2. **Integrity** — `INTEGRITY.md` verifies checksums, repairs or escalates.
3. **Runtime detection** — `router/RUNTIME_DETECTION.md`. Load matching adapter.
4. **Adapter install** — adapter's installation footprint (hooks, commands, context block).
5. **Context detection** — `CONTEXT_DETECTION.md` fingerprints the project.
6. **Calibration** — `CALIBRATION.md` sets escalation threshold from context.
7. **Override** — `OVERRIDE.md` applies manual overrides if present.
8. **Local creation** — ensure `<project>/.ciel/` exists; populate `project.json`; ensure `.gitignore` entry via `GITIGNORE.md`.
9. **Local sync** — `LOCAL_SYNC.md` pulls relevant global learnings as read-only references.
10. **Memory health** — `memory/HEALTH_CHECK.md`.
11. **Git setup** — `GIT_SETUP.md` (only if `~/.ciel/` just created).
12. **Backup** — `BACKUP.md` schedules + takes an initial snapshot if missing.
13. **Announce** — emit a one-paragraph summary + append `activity.log`.

## Idempotency

Re-init does not reset. Every phase is rerunnable and performs:

- presence checks before creation,
- diff-then-repair for modified tracked files,
- preserves user-made content outside Ciel anchor blocks,
- announces any corrections at the end.

## Auto-Invocation

Ciel auto-runs `INIT.md` whenever:

- `~/.ciel/.git` is missing,
- `~/.ciel/INTEGRITY.json` fails verification,
- detected project has no `.ciel/`,
- the host runtime changed since last init.

## Manual

`/ciel-init` runs all phases explicitly; `/ciel-init --local` runs only project-scoped phases 5–9.
