---
locked: true
---

# COUNCIL — Council of Five

Ciel's integration immune system and self-modification gate. Convened automatically on any Council-scope event (see `invocation_scopes/`).

## Members

Five lenses, each a dedicated evaluator:

- **Coherence** — repository harmony (`members/COHERENCE.md`)
- **Capability** — genuine expansion vs redundancy (`members/CAPABILITY.md`)
- **Safety** — risk vectors, veto authority (`members/SAFETY.md`)
- **Efficiency** — leanness, bloat, perf (`members/EFFICIENCY.md`)
- **Evolution** — growth trajectory (`members/EVOLUTION.md`)

## Three Stages

**Stage 1 — Independent scoring (parallel).** Each member evaluates the candidate against their lens. Inputs: artifact (at L1 by default), lens rubric, stage-1 prompt. Output: `{ score: 0..10, rationale, flags, requests }` where `requests: L2` upgrades loading to full artifact for a second pass.

**Stage 2 — Cross-review (anonymous).** Each member sees the four peer outputs without attribution (`ANONYMIZATION.md`). They may revise. Output: `{ score, rationale, challenge_of: [anon_id], delta_reason }`.

**Stage 3 — Chairman synthesis.** Ciel herself synthesizes per `CHAIRMAN.md` and `prompts/council/chairman_synthesis.md`.

## Voting Math

- **Pass**: ≥ 3/5 final scores ≥ pass threshold (`council.config.pass_score`, default 6) AND Safety score > 3 (no veto).
- **Reject**: any of {majority below threshold, Safety ≤ 3, Chairman overrides due to emergent risk}.
- **Deadlock**: exactly 2-2 with one abstention, or scores clustered around threshold ± 1.0. Goes to `ESCALATION.md`.

## Safety Veto — Absolute

A Safety score ≤ 3 is a hard block regardless of other votes. Mirrors `llm-council`'s Accuracy-as-ceiling pattern. Cannot be overridden by Chairman. Cannot be overridden by user via auto-path — requires Constitutional amendment procedure.

## Quorum

- Minimum 3 non-abstaining members for any decision.
- Safety must be present and non-abstaining.
- If Safety is absent/timed-out, the run is re-attempted once; second failure escalates.

## Invocation Scopes

See `invocation_scopes/`. Each scope has its own preamble to the prompts so members know what kind of decision they're evaluating.

## Cost

~10 model calls per Council run (5 Stage 1 + 5 Stage 2). Ciel batches runs where possible and amortizes via prompt caching of personas and rubrics.
