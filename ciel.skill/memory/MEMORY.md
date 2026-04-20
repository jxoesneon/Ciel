# MEMORY — Architecture Master

Ciel's persistent memory. Primary: **MemPalace-rs**. Fallbacks: SQLite, filesystem KV, custom.

## Goals

- Cross-session persistence of registry, routes, traces, Council decisions.
- Per-project isolation (no cross-project bleed).
- Semantic recall via embeddings.
- AAAK (Adaptive Anchored Attention Keys) summarization for context compression.
- Survive Ciel upgrades; migrate on schema bumps.

## Layers

| Layer | Role |
| --- | --- |
| `MEMPALACE.md` | primary backend contract |
| `PARTITION.md` | partition schema (keyspaces) |
| `INSTALL.md` | install + auto-update protocol |
| `FALLBACK.md` | fallback trigger + selection |
| `GLOBAL_STORE.md` | `~/.ciel/` scope |
| `LOCAL_STORE.md` | `.ciel/` scope |
| `MERGE_SEMANTICS.md` | local overrides global, promotion semantics |
| `HEALTH_CHECK.md` | startup verification + corruption recovery |
| `backends/*` | alternative backends |

## Access Pattern

All memory access goes through `seed_skills/mempalace_manager/SKILL.md` (backend-abstracting). Direct backend calls are forbidden.

## Keyspace (top-level)

```text
ciel/
├── registry/                 # indexed skills + metadata
├── route_registry/           # routes + hit stats
├── council/                  # decisions, mappings
├── acquisition/              # per-attempt traces
├── traces/                   # sandbox traces
├── runtimes/                 # probed runtime fingerprints
├── projects/<proj_id>/       # local learnings per project
├── improvements/             # proposals + rollbacks
├── sources/                  # acquisition sources trust
└── meta/                     # schema version, integrity
```

## Schema Versioning

`meta/schema_version` tracks the format. On version bump, `INSTALL.md` runs a migration. Migrations are git-committed and reversible.

## Durability

Writes are fsync'd on critical paths (council decision, registry mutation). Reads may be eventually-consistent for telemetry.

## Observability

Every memory op is optionally telemetered via `observability/OTEL.md` (sampling-controlled).
