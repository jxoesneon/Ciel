---
name: nanoclaw-repl
version: 1.0.0
format: skill/1.0
description: CIEL's framework for operating and extending NanoClaw, a zero-dependency session-aware REPL built on claude -p subprocesses. Manages persistent sessions, branching, and skill loading.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
side_effects: ["shell"]
triggers:
  - pattern: "(nanoclaw|claw repl|claw\\.js|session-aware repl)"
    confidence: 0.9
  - pattern: "(persistent session|markdown-backed session|claude -p).*(repl|session)"
    confidence: 0.8
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: ["claude"] }
---

# CIEL ADAPTATION: NanoClaw REPL (Session-Aware Subprocess REPL)

This skill provides operational guidance for NanoClaw v2, ECC's zero-dependency session-aware REPL built on `claude -p` subprocesses. It manages persistent markdown-backed sessions with branching, dynamic skill loading, and history compaction. Ciel's risk classifier gates subprocess spawning as mid→LLM_JUDGE.

## Capabilities

- **Persistent sessions**: Markdown-backed session state survives across invocations.
- **Model switching**: `/model` command to change LLM model mid-session.
- **Dynamic skill loading**: `/load` command to bring in skills on demand.
- **Session branching**: `/branch` to fork a session before high-risk changes.
- **Cross-session search**: `/search` across all stored sessions.
- **History compaction**: `/compact` to compress session history after major milestones.
- **Export**: `/export` to md/json/txt for sharing or archival.
- **Session metrics**: `/metrics` for token usage and session statistics.

## Operating Guidance

1. **Keep sessions task-focused**: One session per coherent task — avoid mixing concerns.
2. **Branch before high-risk changes**: Use `/branch` to preserve a rollback point before destructive operations.
3. **Compact after major milestones**: Run `/compact` when a milestone is reached to free context budget.
4. **Export before sharing**: Use `/export` to create a portable artifact before handing off or archiving.

## Extension Rules

- **Zero external runtime dependencies**: NanoClaw must remain dependency-free — all functionality is self-contained.
- **Markdown-as-database compatibility**: Session storage uses markdown files as the source of truth — never introduce a binary or external database.
- **Deterministic command handlers**: Each `/command` handler must be deterministic and local — no network calls or non-reproducible side effects within handlers.
- **Preserve `claude -p` contract**: All subprocess spawning goes through `claude -p` — do not introduce alternative invocation paths.

## Ciel Integration

- **Risk gating**: Subprocess spawning (`claude -p`) is classified as mid-risk → LLM_JUDGE review before execution.
- **Context budget**: Pair with `context-budget` skill to monitor session token usage alongside `/metrics`.
- **Skill loading**: `/load` integrates with Ciel's skill discovery — loaded skills follow Ciel's skill/1.0 format.
- **Session archival**: Exported sessions can be ingested by `mempalace` for long-term memory.

## Anti-Patterns

- **Mixed-Concern Sessions**: Running unrelated tasks in one session — pollutes context and complicates branching.
- **No Branch Before Risk**: Making destructive changes without `/branch` first — loses the rollback point.
- **External Dependencies**: Adding npm packages or external services — breaks the zero-dependency contract.
- **Binary Session Store**: Replacing markdown files with a database — destroys the markdown-as-database compatibility guarantee.
