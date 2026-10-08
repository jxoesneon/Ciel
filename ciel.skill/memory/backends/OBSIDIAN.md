# OBSIDIAN — Obsidian Vault Markdown Backend

Plain-text Markdown key-value store optimized for Obsidian. Stored inside an Obsidian vault under `.ciel-brain/`.

## Location & Config

- Vault Path: Configured via `memory.obsidian.vault_path` (default: `~/.ciel/vaults/default`).
- Brain Subfolder: `.ciel-brain/`
- Directory Structure:

  ```text
  <vault_path>/.ciel-brain/
  ├── registry/
  │   └── <key>.md
  ├── council/
  │   └── <key>.md
  └── <partition>/
      └── <key>.md
  ```

## Security & Path Sanitization

To guarantee safety and prevent directory traversal attacks:

1. **Path Validation**: Partition and key identifiers MUST match `^[a-zA-Z0-9_-]+$`. Any identifier containing `..`, `/`, `\`, or null bytes is rejected immediately with a `PathTraversalError`.
2. **Isolation & Permissions**: Vault storage follows `isolation_strict: true`. Internal Ciel partitions are created with `0700` POSIX permissions. Storing Ciel brain partitions in unencrypted public shared storage (e.g. Android public shared documents) is strictly forbidden unless isolated in an encrypted container.

## Storage Format

Each entry is a standard Markdown file with YAML frontmatter:

```markdown
---
key: "key_name"
partition: "registry"
created: 2026-07-15T07:42:00Z
updated: 2026-07-15T07:42:00Z
tags: ["ciel", "metadata"]
---

# <Key Name>

<Value content goes here>
```

## API Mapping

| MemPalace API | Obsidian Impl |
| --- | --- |
| `put(partition, key, value)` | Validate paths; write YAML frontmatter + markdown body to `.ciel-brain/<partition>/<key>.md` |
| `get(partition, key)` | Validate paths; read and parse YAML frontmatter + body from file |
| `query(partition, filter)` | Parse frontmatter of files in partition directory; match metadata filters |
| `search(partition, query, top_k)` | Use `obsidian-hybrid-search` (hybrid vector/BM25); fallback to frontmatter tag index + `ripgrep` |
| `delete(partition, key)` | Validate paths; unlink file `.ciel-brain/<partition>/<key>.md` |
| `list(partition, prefix)` | Validate paths; list `.md` files matching `<prefix>*` in partition directory |
| `compact(partition)` | Re-index frontmatter tags, prune orphaned metadata, optimize frontmatter cache |
| `snapshot(partition, path)` | Archive partition directory to snapshot file |

## Search Mechanics

When `obsidian-hybrid-search` service is present, search utilizes vector embeddings alongside BM25 lexical ranking. When unavailable, search degrades to frontmatter tag lookup combined with bounded `ripgrep` scanning across partition files.

## Performance Optimization

To minimize overhead on mobile and FUSE filesystems:

- In-memory frontmatter metadata cache avoids redundant disk I/O on repetitive `query`/`list` operations.
- File system watchers update the metadata index incrementally.
