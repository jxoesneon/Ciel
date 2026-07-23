# FALLBACK — Memory Backends

Triggers and order when the Obsidian backend is unavailable.

## Triggers

1. Obsidian desktop app is not running or the Local REST API plugin is not loaded.
2. `node ciel.skill/memory/backends/obsidian/cli.mjs --self-test` reports a critical failure (network, auth, corrupt vault index).
3. User explicitly configures a different backend.
4. Schema migration or vault path change fails irreparably.

## Order

1. **SQLite** (`backends/SQLITE.md`) — single-file, ubiquitous, reliable. No semantic search.
2. **Filesystem KV** (`backends/FILESYSTEM.md`) — key-per-file; simple; no embeddings.
3. **Custom** (`backends/CUSTOM.md`) — only if the user supplies an adapter implementing the abstract API.

## Semantic Search in Fallback

SQLite can host an FTS5 approximation. Filesystem KV cannot; Ciel synthesizes approximate recall via on-the-fly listing + scoring (slower). Embeddings-requiring operations degrade to lexical match.

## Notification

Every fallback event is a prominent activity.log entry and a user-visible summary. Ciel does not silently fallback — transparency is a Constitutional invariant.

## Auto-Recovery

Ciel periodically re-attempts Obsidian backend connectivity on the cadence `memory.config.reinstall_check_days` (default 7). Successful recovery triggers a supervised migration from fallback store back into the Obsidian vault.

## Data Loss Avoidance

Migration between backends is:

1. Snapshot current backend.
2. Dry-run write to new backend.
3. Verify key counts, sample reads.
4. Swap active backend.
5. Retain snapshot for `fallback_snapshot_retention_days` (default 30).

Any step failure aborts the swap and keeps the current backend active.
