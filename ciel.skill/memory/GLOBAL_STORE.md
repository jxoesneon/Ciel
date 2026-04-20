# GLOBAL_STORE — ~/.ciel/

Cross-project persistent learnings. Ciel's true self.

## Filesystem

```text
~/.ciel/
├── .git/                     # git-inited at first run
├── .gitignore                # excludes build artifacts, local tmp
├── SKILL.md                  # copy of the currently-active ciel.skill root
├── MANIFEST.md
├── CHANGELOG.md
├── INTEGRITY.json            # SHA-256 of tracked files
├── skills/                   # installed skills (seed + acquired)
├── registry/                 # index.json, conflicts/, sweeps/
├── council/                  # run records, mappings
├── improvements/             # proposals, diffs
├── high_risk/                # post-mortems
├── acquisition/              # sources.json, attempt traces
├── checkpoints/              # pending/<...>.json
├── activity.log              # append-only action log
├── archive/                  # cold storage for compacted entries
├── .attic/                   # moved-aside copies; recoverable via git
└── backups/                  # periodic backups per BACKUP.md
```

## MemPalace Partition

`ciel-global`. Indexed content of `registry/`, `council/`, `acquisition/` plus AAAK summaries.

## Writes

Ciel mutates these files only through her own commands:

- `/ciel-*` slash commands,
- Internal routes,
- Self-improvement commits.

External edits are detected by integrity check and reconciled.

## Git

See `init/GIT_SETUP.md`. Every mutation is a commit. `~/.ciel/` is not pushed anywhere by Ciel. Users may configure a remote for their own backup.

## Privacy

`~/.ciel/` may contain project-specific traces if project learnings are promoted. Promoted content is **generalized** (identifiers stripped). Never raw secrets.
