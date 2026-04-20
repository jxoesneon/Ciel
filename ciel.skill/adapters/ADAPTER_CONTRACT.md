# ADAPTER_CONTRACT

The abstract minimum a host runtime must expose for Ciel to function.

## Required Capabilities (the Floor)

A conformant runtime MUST provide:

1. **SKILL.md loading** — can load a skill's root `SKILL.md` and surface it to the model's context.
2. **Subagent delegation** — some markdown-file-defined delegation primitive (nested or parallel).
3. **MCP client** — can connect to and invoke Model Context Protocol servers.
4. **Shell execution** — can execute arbitrary shell commands and capture stdout / stderr / exit code.
5. **File I/O** — can read, write, and stat files within a project root and within `$HOME`.
6. **Context file** — supports a persistent per-project instruction file (e.g. `CLAUDE.md`, `GEMINI.md`).

Without these six, Ciel cannot operate. Her Generic adapter's first action is to verify the floor.

## Recommended Capabilities (enhance, not required)

1. Lifecycle hooks (pre/post tool).
2. Parallel subagent dispatch.
3. Plan / dry-run mode.
4. Permission rules (allow/deny).
5. Checkpoint / resume.
6. Prompt / token caching.
7. Native telemetry (OTEL or equivalent).
8. Multimodal generation (images, video, audio).
9. Computer use (UI automation).

Each recommended capability maps to a *route* in the Master Router that Ciel will prefer when available (see `router/ROUTE_REGISTRY.md`).

## Interface Shape

Each adapter exposes the following logical functions to Ciel (pseudocode):

```text
adapter:
  load_skill(path)                             -> Skill
  spawn_subagent(name, input, parallel=false)  -> Handle
  invoke_mcp(server, tool, args)               -> Result
  shell(cmd, cwd, env, timeout)                -> { stdout, stderr, code }
  fs_read(path)  |  fs_write(path, content)
  context_inject(scope=global|project|local, content)
  hook_register(event, handler)                   # optional
  plan_mode(enabled: bool)                        # optional
```

Runtime-specific adapters under `adapters/<id>/` translate these calls to native runtime primitives.

## Detection → Load

See `router/RUNTIME_DETECTION.md`. Based on detection, Ciel loads exactly one adapter per session. Adapters are hot-swappable only across sessions.

## Generic Adapter Fallback

If a runtime is unknown, Ciel loads `adapters/generic/` and runs `adapters/generic/CAPABILITY_PROBE.md`. If the probe confirms the floor, Ciel operates — degraded but functional — while a background self-improvement task builds a dedicated adapter via `seed_skills/runtime_adapter_builder/SKILL.md`.

## Contract Testing

Each adapter ships with capability flags in its `ADAPTER.md` and a self-test that Ciel runs at init. Failures are logged; degraded-mode routes are substituted automatically.
