# LOCAL_SYNC — Global → Local Seeding

At local init, Ciel pulls relevant global learnings down as read-only references for this project.

## What Gets Synced

- Global rules matching the project's detected language / framework.
- Skills tagged compatible with the detected runtime.
- Generalized promoted learnings from semantically similar past projects.

## Read-Only

Synced references are linked, not copied-and-modifiable. Stored under `<project>/.ciel/refs/` with symlinks or pointer files:

```text
<project>/.ciel/refs/
├── skills/
│   └── git.ref            # points at ~/.ciel/skills/git/
├── rules/
│   └── rust-common.ref
└── learnings/
    └── monorepo-patterns.ref
```

Pointer files store a resolved path + a version pin. Modifying a referenced skill propagates to all projects referencing it. Ciel never writes through a `.ref`.

## Query

```text
cielref resolve skills/git → ~/.ciel/skills/git/SKILL.md@1.2.0
```

Surface exposed via `seed_skills/filesystem/SKILL.md` with a `--follow-refs` flag on reads.

## Updating Pins

On global update of a referenced skill, Ciel:

1. detects via integrity sweep,
2. for each project with pointer, checks compatibility diff,
3. if compatible, updates the pin automatically,
4. if breaking, surfaces as an improvement proposal per project.

## User Opt-Out

```yaml
local_sync:
  enabled: true
  categories: [rules, skills, learnings]
```

Disabling keeps `.ciel/` pure, but the project loses the benefit of Ciel's accumulated knowledge.
