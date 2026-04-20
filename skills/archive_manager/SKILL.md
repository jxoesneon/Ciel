---
name: archive_manager
version: 1.0.0
description: ZIP / tar / gz operations — create, extract, inspect (.skill file handling).
triggers: [zip, tar, archive, unzip, extract]
tags: [fs, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md] }
---

# archive_manager

Archive create / extract / inspect.

## Operations

- `archive.zip(paths, out)` / `archive.unzip(path, out_dir)`.
- `archive.tar(paths, out, compress=gz|zst|xz)` / `archive.untar(path, out_dir)`.
- `archive.inspect(path)` — list entries with sizes, mtimes, hashes.
- `archive.skill_pack(skill_dir)` — produces `.skill` (ZIP) with checksum sidecar.
- `archive.skill_unpack(path, dest)`.

## I/O Contract

```yaml
io_contract:
  input: { op, path_or_paths, "out?" }
  output: { path_or_entries }
  idempotent: for_same_inputs
  side_effects: [fs]
```

## Safety

- Extraction refuses entries escaping the target directory (path-traversal defense).
- Maximum entry count and size enforced.
