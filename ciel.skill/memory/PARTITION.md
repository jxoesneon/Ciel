# PARTITION — Schema

## Global Partition

`ciel-global` — Ciel's cross-project self.

Keyspace (see `memory/MEMORY.md` for top-level).

## Per-Project Partitions

`ciel-project-<hash>` where `<hash>` is `sha256(abspath(project_root))[:16]`. Created at local init.

Keyspace inside a project partition:

```text
learnings/        # project-specific patterns
rules/            # detected and codified project rules
traces/           # executions scoped to this project
improvements/     # local improvement proposals
context/          # project context snapshots
checkpoints/      # session resume state
```

## Isolation Guarantees

Constitutional (see `core/CONSTITUTION.md` + `domains/ISOLATION.md`):

- No global partition read may be triggered by project-scoped code without a deliberate "lift" call.
- No project partition may be read by another project partition.
- Enforced by `seed_skills/mempalace_manager/SKILL.md` via partition-name scoping on every operation.

## Naming Conventions

- Keys are path-style: `traces/<ts>/<op_id>`.
- Values are JSON or AAAK blobs; metadata is a small JSON object with `kind`, `tags`, `created`.
- No key collisions — enforced via CAS on `put`.

## Purging

When a project is deleted or explicitly purged:

```text
ciel purge <project_root>
```

The partition is dropped. No residue in global. Confirmed by health check.

## Size Management

Per-partition size limits in `configuration/global/memory.config.md`. Oldest/lowest-value entries are AAAK-compacted first; truly old entries are moved to cold archive (`~/.ciel/archive/`).
