# SUBAGENTS — opencode

OpenCode exposes subagents through the **task** tool, which spawns a child
session with fresh context and returns its result to the parent.

## Primitives

| Need | Mechanism |
| --- | --- |
| Delegate a scoped task | `task` tool with a prompt; child session |
| Parallel fan-out | Multiple concurrent `task` tool calls (one per child) |
| Read-only exploration | `explore` subagent |
| General work | `general` subagent |
| Background command | `commands/*.md` with `subagent: true` |

## The Floor

`ADAPTER_CONTRACT.md` requires a markdown-file-defined delegation primitive.
OpenCode satisfies it with:
- built-in agents (`build`, `plan`, `explore`, `general`),
- custom agents from `~/.config/opencode/agent/*.md` and `.opencode/agent/*.md`,
- the Ciel **Guilds** (`agents/*.md` in the source repo) registered as agents.

## Concurrency

OpenCode runs concurrent task dispatches; the practical ceiling for the Council
of Five is five parallel children. See `COUNCIL_INVOCATION.md` for the exact
two-stage + chairman topology and the `spawn_receipts.json` evidence contract.

## Evidence

Every non-trivial delegation records the child session/agent id in the run
artifact so isolation is verifiable — never an inline "pretend" deliberation.
