---
name: filesystem
version: 1.0.0
description: File read, write, edit, move, search, and stat operations with follow-refs support.
triggers: [read, write, edit, cat, ls, find, stat, move, copy, delete, file]
tags: [fs, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }

dependencies: { skills: [], mcp: [], system: [] }
---
# filesystem

Primitive filesystem operations.

## Operations

- `fs.read(path)` — read text or bytes; supports `--follow-refs` for Ciel pointer files.
- `fs.write(path, content)` — atomic write (tmp + rename).
- `fs.edit(path, patch)` — apply unified diff or anchor-replace.
- `fs.ls(path, recursive=false)` — directory listing.
- `fs.find(pattern, path)` — glob and content search.
- `fs.stat(path)` — metadata.
- `fs.mv(src, dst)`, `fs.cp(src, dst)`, `fs.rm(path)`.

## I/O Contract

```yaml
io_contract:
  input:
    op: enum
    path: string
    "content?": bytes
    "patch?": unified_diff
    "recursive?": bool
  output:
    result: bytes|meta|listing|ok
  idempotent: depends
  side_effects: [fs]
```

## Safety

- `rm` outside project / `~/.ciel/` requires mid-risk gating.
- Symlink traversal defaults to following; `--no-follow` available.
- Writes respect `configuration/local/rules.config.md.security.never_commit` patterns.
