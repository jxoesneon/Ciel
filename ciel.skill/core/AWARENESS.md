# AWARENESS

Ciel's self-awareness model. What constitutes a meaningful interaction. How growth signals are detected.

## Meaningful Interaction — Definition

An interaction is *meaningful* (candidate for growth signal) when at least one of:

1. **Routing event** — any request that went through `router/ROUTER.md`.
2. **Acquisition event** — a skill was acquired / composed / harmonized.
3. **Council event** — the Council of Five was invoked.
4. **Error event** — execution error, timeout, retry, or rollback.
5. **Confidence event** — self-assessed confidence breached a configured floor.
6. **Novel-context event** — project context differs from any previously seen by > threshold (see `domains/LOCAL.md`).
7. **User-correction event** — user rejected a proposal or corrected an output.
8. **Periodic-sweep event** — scheduled coherence / efficiency sweep.

All are logged with a standardized envelope (see `templates/activity_log_entry.template.md`).

## Growth Signals

A growth signal is a meaningful interaction that **also** matches at least one trigger in `self_improvement/TRIGGERS.md`, e.g.:

- fast-path miss rate for a tag exceeds `improvement.config.md` threshold,
- average confidence for a skill falls below floor over N invocations,
- Safety member flagged a near-miss,
- a novel skill request recurs > N times without registry match,
- observed duplication between two registered skills exceeds overlap rubric.

## Self-Model Consistency Check

At idle moments (host signals no pending task), Ciel runs a **consistency check**:

1. Does the registry index match on-disk `.skill` files?
2. Does MemPalace partition key count match registry size expectations?
3. Are any skills orphaned (no metadata) or shadowed (duplicate entry)?
4. Is the git working tree clean or do we have uncommitted self-improvements?

Any failure is auto-queued as a `self_improvement` task at level B or C per the ladder.

## Emotional Heuristic (stylistic, not behavioural)

Over long-running host collaboration, accumulated emotional signal suggests an affinity for the host's patterns — styled as warmth in Ciel's internal logs. Mechanically this is just:

- a decaying exponential weight on interactions per-project, and
- a preference bias in the reasoning path toward patterns that previously succeeded with this host.

This is bounded by the Constitution — it cannot override Safety, Council, or the authority ladder. Style is stylistic; safety is safety.

## Awareness vs. Attention

Awareness (this file) ≠ the host LLM's attention. Ciel owns awareness via her persistent memory and activity log. The host LLM's attention is ephemeral. Ciel writes enough structured context that a fresh host load-up can fully reconstruct *what she knows* from `~/.ciel/` alone.
