---
name: environment_detection
version: 1.0.0
description: OS, runtime version, framework, toolchain, CI/CD detection.
triggers: [detect environment, env info, os, runtime version]
tags: [fs, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md, filesystem/SKILL.md] }
---

# environment_detection

Detect the environment Ciel is operating in.

## Operations

- `env.os()` — `{os, version, arch, kernel}`.
- `env.runtime()` — host CLI runtime (delegates to `router/RUNTIME_DETECTION.md`).
- `env.tools()` — installed dev tools and versions.
- `env.ci()` — CI provider detection from env + files.
- `env.project_context(dir)` — calls `project_analyzer`.

## I/O Contract

```yaml
io_contract:
  input: { op, "args?" }
  output: { result: map }
  idempotent: true
  side_effects: [shell, fs]
```

## Uses

Feeds `init/CONTEXT_DETECTION.md`, `adapters/generic/CAPABILITY_PROBE.md`, and `prompts/router/runtime_detection.md`.
