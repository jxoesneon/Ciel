# ADAPTER — Generic

For any runtime that is not Claude Code or Gemini CLI. Functional at the floor level; enhancements are researched and added as dedicated adapters over time.

## When Loaded

`router/RUNTIME_DETECTION.md` returns `id: generic` when:

- no known environment variables are present,
- no known filesystem fingerprints exist,
- binary probes fail.

## First Action

Runs `adapters/generic/CAPABILITY_PROBE.md` to confirm the floor from `adapters/ADAPTER_CONTRACT.md`:

1. SKILL.md loading
2. Subagent delegation
3. MCP client
4. Shell execution
5. File I/O
6. Context file

If any floor capability is absent, Ciel operates in **minimal mode**: fast path and shell-only reasoning, no acquisition, no Council. This is a degraded but safe state.

## Research Protocol

Concurrently, Ciel enqueues a background self-improvement task to build a dedicated adapter. See `adapters/generic/RESEARCH_PROTOCOL.md` and `seed_skills/runtime_adapter_builder/SKILL.md`.

## Contract Implementation

Generic adapter translates adapter-contract functions to reasonable defaults:

| Function | Generic implementation |
| --- | --- |
| `load_skill(path)` | host-native or read `SKILL.md` as markdown system prompt |
| `spawn_subagent(name, input)` | if subagents unsupported, inline the persona in-context (serial only) |
| `invoke_mcp(server, tool, args)` | if unsupported, invoke via HTTP if MCP server exposes one |
| `shell(cmd, ...)` | direct pipe to `/bin/sh` via child process |
| `fs_read/write(path)` | direct filesystem ops |
| `context_inject(...)` | append to a `CONTEXT.md` at scope |
| `hook_register(...)` | not supported; return stub |
| `plan_mode(enabled)` | not supported; return stub |

## Degraded Features

Without hooks: no automatic pre-flight gating → Ciel inlines the check before each tool call (more tokens, same safety).

Without parallel subagents: Council runs **sequentially**, which is slower but correct.

Without plan mode: Ciel uses an in-LLM "describe before execute" prompt pattern as a softer equivalent.

## Exit

Once the research protocol produces a dedicated adapter, it is Council-gated and installed; Ciel switches to it in the next session.
