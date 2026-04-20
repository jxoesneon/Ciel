# FILESYSTEM — Fallback KV Backend

Plain-file key-value store. Ultimate fallback when nothing else is available.

## Layout

```text
~/.ciel/fs_backend/
├── ciel-global/
│   ├── registry/
│   │   └── <url-encoded-key>.json
│   ├── council/
│   └── ...
└── ciel-project-<hash>/
    ├── learnings/
    └── ...
```

Partition = top-level directory. Keys are url-encoded to survive filesystem constraints.

## Value Files

- JSON for structured entries.
- `.aaak` for AAAK blobs.
- `.meta.json` sidecar with `{created, updated, kind, tags}`.

## API Mapping

| MemPalace API | FS impl |
| --- | --- |
| `put` | write tmp → rename (atomic) |
| `get` | read file |
| `query` | listdir + filter |
| `search` | listdir + lexical scoring (slow at scale) |
| `delete` | unlink |
| `list(prefix)` | `find <partition>/<prefix>*` |
| `compact` | AAAK re-compression pass across large values |
| `snapshot` | `tar cf` of partition |

## Locking

File-level advisory locks (`flock`) for writes. Reads lock-free.

## Performance

Acceptable up to ~10k entries per partition. Beyond that, Ciel nags the user to reinstate MemPalace or SQLite.

## Degraded Features

- No native semantic search.
- No transactions.
- No FTS5.
- Search and queries at scale are O(N).

## Use Case

Viable for ephemeral environments (containers, sandboxes, brand-new machines without toolchains). Ciel never recommends this as steady-state.
