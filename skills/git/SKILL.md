---
name: git
version: 1.0.0
description: Git init, commit, diff, log, revert, branch, stash, remote.
triggers: [git, commit, diff, log, branch, revert, stash]
tags: [vcs, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md], system: [git] }
---

# git

Git operations.

## Operations

- `git.init(dir)`, `git.clone(url, dir)`
- `git.add(paths)`, `git.commit(msg, trailers?)`
- `git.diff(ref?, --staged?)`, `git.log(n, path?)`
- `git.revert(sha)`, `git.reset(mode, ref)`  (reset is mid+ risk)
- `git.branch(name, from?)`, `git.checkout(ref)`
- `git.stash(save|pop|list)`
- `git.push(remote, branch)` — mid risk.
- `git.push --force` — critical risk.
- `git.remote(list|add|remove)`

## I/O Contract

```yaml
io_contract:
  input: { op: string, args: map }
  output: { result: map }
  idempotent: depends
  side_effects: [fs, "network?"]
```

## Integration

Used by `~/.ciel/` git history, `self_improvement/ROLLBACK.md`, and `registry/VERSIONING.md`.
