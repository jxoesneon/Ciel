---
locked: true
---

# MEMPALACE — Primary Backend

`locked: true` in the sense that the primary-backend contract is Constitutional; the implementation is upgradable.

MemPalace-rs is a high-performance local memory store with AAAK compression, semantic search, and multi-partition isolation. Ciel uses it as her default.

## Why MemPalace-rs

- Local-first; no external deps at runtime after install.
- AAAK format for context-efficient recall.
- Native partition model for cross-project isolation.
- Rust-native performance; embeds cleanly in CLI workflows.
- Open source; Ciel can keep it at latest via cargo.

## API Contract (abstract)

Wrapped by `seed_skills/mempalace_manager/SKILL.md`:

```text
put(partition, key, value, metadata)
get(partition, key)
query(partition, filter)                     # structured key pattern
search(partition, query, top_k)              # semantic
delete(partition, key)
list(partition, prefix)
compact(partition)                           # AAAK recompression sweep
snapshot(partition, path)                    # backup
restore(partition, path)                     # restore
stats(partition)
```

## Partition Model

See `PARTITION.md`. Ciel uses a **dedicated partition** for every project plus one global partition, so no cross-project bleed.

## AAAK

Adaptive Anchored Attention Keys: entries can be stored as compressed summaries that expand on retrieval at high fidelity. Ciel stores:

- long traces as AAAK summaries,
- council rationales as AAAK blobs,
- research findings as AAAK,
- project context summaries as AAAK.

## Health

`HEALTH_CHECK.md` defines the startup sequence: ping, schema check, partition list verification, and a read-write self-test. Any failure auto-falls-back per `FALLBACK.md`.

## Versioning

`INSTALL.md` keeps MemPalace-rs at latest compatible. Schema migrations on upgrades are gated by integrity checks.
