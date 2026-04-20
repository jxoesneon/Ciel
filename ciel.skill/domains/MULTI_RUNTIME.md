# MULTI_RUNTIME

When Claude Code and Gemini CLI both invoke Ciel on the same project.

## Scenario

A developer uses Claude Code in one terminal and Gemini CLI in another, both on the same project. Both load `ciel.skill`. Both must agree on state.

## State Ownership

- `~/.ciel/` is shared. Single source of truth.
- `<project>/.ciel/` is shared. Single source of truth.
- MemPalace partitions are shared.
- `activity.log` is append-only and shared.

## Concurrency

- Writes use file-level locks (`flock`) on `*.json` indices.
- MemPalace-rs provides its own concurrency primitives; when available, used.
- For git commits to `~/.ciel/`, a file lock on `~/.ciel/.git/ciel.lock` serializes Ciel's self-mutations across runtimes.
- Long-running operations (acquisition, Council) take an exclusive lock on their subsystem.

## Conflict Resolution

When two runtimes propose overlapping self-improvements:

- Second-arriving proposal is rebased on first.
- If rebase conflicts, the later proposal is queued and Ciel batches them at the next cadence sweep.

## Per-Runtime Hooks

Each adapter installs its own hooks. They do not conflict because they use runtime-specific file paths (`.claude/hooks/` vs `.gemini/hooks/`). Context files (`CLAUDE.md`, `GEMINI.md`) are per-runtime and share only the project rules that are runtime-agnostic.

## Runtime Preference

If both runtimes are active and the user doesn't specify, route preferences:

- Computer use → Claude Code.
- Multimodal generation → Gemini CLI.
- Parallel subagents → Gemini CLI (native).
- Ultraplan → Claude Code.
- Everything else → whichever session issued the request.

## Observability

`activity.log` includes a `runtime` field per entry, so users can see which runtime did what.
