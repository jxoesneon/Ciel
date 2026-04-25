# LOCAL_STORE — .ciel/

Project-scoped domain. Created at local init; added to the project's `.gitignore`.

## Filesystem

```text
<project>/.ciel/
├── project.json              # detected context + project fingerprint
├── rules/                    # codified project-specific rules
├── overrides/                # config overrides for this project
├── skills/                   # project-scoped skills (if any)
├── learnings/                # raw learnings; candidates for promotion
├── traces/                   # per-session traces, sandbox runs
├── checkpoints/              # resume state per session
├── activity.log              # local log mirror + project-scoped entries
└── cache/                    # disposable caches (not tracked)
```

## MemPalace Partition

`ciel-project-<hash>` where `<hash> = sha256(abspath(project))[:16]`.

## .gitignore Entry

Appended by `init/GITIGNORE.md`:

```text
# Ciel local domain
.ciel/
```

If a `.gitignore` already exists, the entry is added once (idempotent). If missing, a minimal `.gitignore` is created.

## Isolation

Strict. The local store may not:

- read global partition except through `MERGE_SEMANTICS.md` lift operations,
- reference another project's paths,
- write outside the project root,
- be read by a different project's Ciel instance (partition scoping enforces).

## Cleanup

On project delete:

```text
ciel purge <project>
```

removes the `.ciel/` directory, drops the MemPalace partition, and removes any global promotion breadcrumbs referencing the project.

## Project Fingerprint

`project.json` contains:

```json
{
  "id": "<hash>",
  "root": "/absolute/path",
  "detected_at": "...",
  "language": "...",
  "framework": "...",
  "runtime_hint": "claude_code|gemini_cli|generic",
  "escalation_override": null
}
```

Updated only by `init/CONTEXT_DETECTION.md` and `init/CALIBRATION.md`.
