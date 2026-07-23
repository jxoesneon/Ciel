---
locked: true
---

# MEMPALACE — Primary Backend Contract

`locked: true` in the sense that the primary-backend contract is Constitutional; the implementation is upgradable.

The Obsidian backend (`ciel.skill/memory/backends/obsidian/`) is the default memory substrate for Ciel on the `Obsidian` branch. It provides a local-first, markdown-native vault with semantic search, knowledge graph traversal, and multi-partition isolation.

## Why Obsidian

- Local-first; no external deps at runtime after the Obsidian app and adapter dependencies are installed.
- Human-readable, editable markdown files; every durable value can be inspected by the user.
- Native semantic + hybrid search via `obsidian-hybrid-search`.
- Graph traversal via `obra/knowledge-graph`.
- Open vault format; any AI agent with REST or MCP access can read and write the same brain.

## API Contract (abstract)

Wrapped by `skills/obsidian-memory/SKILL.md`:

```text
put(partition, key, value, metadata)
get(partition, key)
query(partition, filter)                     # structured key pattern
search(partition, query, top_k)              # semantic/hybrid
delete(partition, key)
list(partition, prefix)
compact(partition)                           # re-index hybrid search + knowledge graph
snapshot(partition, path)                    # backup
restore(partition, path)                     # restore
stats(partition)
```

## Partition Model

See `PARTITION.md`. Ciel uses a **dedicated vault folder** for every project plus one global scope, so no cross-project bleed.

## Storage Format

Values are stored as markdown files with YAML frontmatter. Long binary values are base64-encoded (`_ciel_enc: base64`). Large contexts are synthesized into dense wiki pages rather than AAAK binary blobs.

## Health

`HEALTH_CHECK.md` defines the startup sequence: ping the Local REST API, vault path check, partition list verification, and a read-write self-test. Any failure auto-falls-back per `FALLBACK.md`.

## Versioning

`INSTALL.md` keeps the Obsidian adapter dependencies up to date. Schema migrations on upgrades are gated by integrity checks.
