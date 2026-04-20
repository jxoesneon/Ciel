---
name: shell
version: 1.0.0
description: Shell command execution with process management, env vars, piping, timeouts.
triggers: [shell, bash, run, exec, command]
tags: [shell, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
---

# shell

Execute shell commands.

## Operations

- `shell.run(cmd, cwd, env, timeout)` — capture stdout/stderr/exit.
- `shell.pipe([cmd, cmd])` — piped sequences.
- `shell.bg(cmd)` — background (returns handle; managed by adapter).
- `shell.which(bin)` — PATH lookup.

## I/O Contract

```yaml
io_contract:
  input: { cmd: string, "cwd?": path, "env?": map, "timeout_s?": int }
  output: { stdout: string, stderr: string, exit: int }
  idempotent: false
  side_effects: [shell]
```

## Safety

- Honours host permission rules (see `adapters/*/PERMISSIONS.md`).
- Refuses commands matching `configuration/local/rules.config.md.forbidden_ops`.
- `sudo` + privileged binaries are always mid+ risk.
- Timeouts default to `acquisition.config.sandbox_limits.wall_s` in sandbox contexts.
