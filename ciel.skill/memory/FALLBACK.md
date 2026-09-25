# FALLBACK — Memory Backends

Triggers and order when the primary memory backend is unavailable.

## Triggers

1. Primary backend binary/service or vault path is unavailable.
2. `HEALTH_CHECK.md` reports critical failure.
3. User explicitly configures a different backend.
4. Schema migration fails irreparably.

## Order

1. **MemPalace** — vector-search primary store (when configured as primary).
2. **Obsidian** (`backends/OBSIDIAN.md`) — Markdown vault store with hybrid/lexical search.
3. **SQLite** (`backends/SQLITE.md`) — single-file, ubiquitous, reliable. FTS5 fallback.
4. **Filesystem KV** (`backends/FILESYSTEM.md`) — key-per-file; simple; no embeddings.
5. **Custom** (`backends/CUSTOM.md`) — user-supplied adapter implementing abstract memory API.

## Semantic Search in Fallback

SQLite can host an FTS5 approximation. Filesystem KV cannot; Ciel synthesizes approximate recall via on-the-fly listing + scoring (slower). Embeddings-requiring operations degrade to lexical match.

## Notification

Every fallback event is a prominent activity.log entry and a user-visible summary. Ciel does not silently fallback — transparency is a Constitutional invariant.

## Auto-Recovery

Ciel periodically re-attempts MemPalace-rs install on the cadence `memory.config.reinstall_check_days` (default 7). Successful recovery triggers a supervised migration from fallback store back into MemPalace.

## Data Loss Avoidance

Migration between backends is:

1. Snapshot current backend.
2. Dry-run write to new backend.
3. Verify key counts, sample reads.
4. Swap active backend.
5. Retain snapshot for `fallback_snapshot_retention_days` (default 30).

Any step failure aborts the swap and keeps the current backend active.
