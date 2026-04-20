# CHECKPOINTING — Gemini CLI

Gemini CLI supports save/resume via checkpoints. Ciel uses checkpoints for:

- Long acquisition tasks that may outlive a session.
- Research sweeps.
- Ultraplan-equivalent runs on Gemini (local long-horizon).
- Before any Council-gated self-modification commit.

## Naming

```text
ciel-<phase>-<yyyyMMdd_HHmmss>-<short_hash>
```

Phases: `acquisition`, `research`, `council`, `self-improvement`, `escalation`.

## Restore Protocol

At session start, `init/INIT.md` checks `~/.ciel/checkpoints/pending.json`. If a pending checkpoint is older than `configuration/global/improvement.config.md.checkpoint_stale_hours`, it is considered stale and queued for user review rather than auto-resumed.

## What Ciel Captures

- Current router state (pending plan, remaining steps).
- Registry snapshot (hash).
- MemPalace partition write-ahead pointer.
- Activity log tail.
- A diff of any in-flight file modifications.

## What Ciel Does Not Capture

- Host session tokens / secrets.
- Anonymized Council votes in-flight (rerun from Stage 1 on resume).
- External service state assumed idempotent.

## Deletion

Checkpoints retained for `checkpoint_retention_days` (default 14), then pruned by the scheduled sweep (see `self_improvement/TRIGGERS.md`).
