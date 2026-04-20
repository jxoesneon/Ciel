# LOCAL — `<project>/.ciel/`

See `memory/LOCAL_STORE.md` for filesystem layout.

## What Lives Here

- `project.json` — detected context + fingerprint.
- `rules/` — project rules (extracted + codified).
- `overrides/` — config overrides scoped to this project.
- `refs/` — read-only pointers to global resources (from `init/LOCAL_SYNC.md`).
- `skills/` — local-only skills, if any.
- `learnings/` — raw project learnings.
- `traces/` — per-session traces.
- `checkpoints/` — session resume state.
- `escalation.json` — auto-detected + optional override.
- `activity.log` — local log mirror.

## Gitignored

Entire `.ciel/` is git-ignored. `init/GITIGNORE.md` ensures this.

## Idempotent Creation

Running init on a project with existing `.ciel/` triggers integrity verification (`init/INTEGRITY.md`) rather than reset. User-edited files outside Ciel anchor blocks are preserved.

## Scope Rules

- Writes stay inside `<project>/` and `<project>/.ciel/`.
- Reads from global only via explicit `lift()`.
- No writes to other projects' partitions.

## Deletion

`ciel purge <project>`:

1. Drops MemPalace partition `ciel-project-<hash>`.
2. Removes `<project>/.ciel/`.
3. Removes project breadcrumbs from global promotion records.
4. Keeps `.gitignore` entry (harmless).

## Project Fingerprint

Computed as `sha256(abspath(project_root))[:16]`. Note: moving the project to a new path generates a new fingerprint. Ciel detects "same project under a new path" via git remote or package manifest signature and can re-link (user-confirmed).
