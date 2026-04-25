---
name: docker
version: 1.0.0
description: Docker / Compose — build, run, inspect, cleanup, sandbox support.
triggers: [docker, container, compose, podman]
tags: [deploy, scope:both, runtime:any, risk:mid]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md], system: [docker] }
---

# docker

Container operations.

## Operations

- `docker.build(dockerfile, tag, context?)`
- `docker.run(image, args?, mounts?, env?, network?)` — foreground or detached.
- `docker.ps()`, `docker.logs(id)`, `docker.stop(id)`, `docker.rm(id)`.
- `docker.inspect(id)`.
- `compose.up(file)`, `compose.down(file)`.
- `docker.sandbox(image, cmd)` — isolated run used by `acquisition/SANDBOX.md`.

## I/O Contract

```yaml
io_contract:
  input: { op, args: map }
  output: { result }
  idempotent: depends
  side_effects: [shell, fs, network, state_mutation]
```

## Safety

- `--privileged` requires high risk Council.
- Mounting sensitive paths (`/`, `$HOME`, secrets) veto at Safety member if observed in acquired skills.
- Sandboxing default profile: no-network, read-only fs except workdir.
