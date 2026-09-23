# ADR — System-1 Lattice: deep integration of the Jev-protocol decision tier

Status: approved by council docket `council-20260923-system1-deep-integration`
Date: 2026-09-23

## Context

The pre-tool risk gate gained a System-1 shadow tier (`risk/SYSTEM1.md`): a
detached subprocess queries a Jev-protocol endpoint (local `laya-serve` on
127.0.0.1:8765, hosted Jev/AutoJev swappable via env) and logs a typed
`safe/dangerous` verdict per tool call. Strictly advisory.

That deployment answers one question on one surface. The same primitive —
typed `choice`/`noul`/`score` questions over compact state, answered with
probabilities in one forward pass — is applicable to most of Ciel's
classification surfaces. This ADR defines how it spreads without becoming a
second, weaker authority.

## Decision

### Core layer — promote the client to a shared library

New `init/hooks/lib/system1.py`, extracted from `risk_policy.py`:

- `ask_choice(state, key, options)` / `ask_noul(state, key, proposition)` /
  `ask_score(state, key, rubric)` — typed primitives over the Jev protocol.
- `ask_async(payload, surface)` — generalized detached dispatcher
  (`system1.py --ask` reads stdin, posts, logs). All callers share it.
- **Response cache**: `sha256(state+questions)` → `system1/cache/`; repeated
  commands never re-infer.
- **Concurrency bound**: max 2 in-flight shadow processes; saturation drops
  the request (fail-open), never queues unboundedly. Directly fixes the
  burst-spawn hole in the current per-call `Popen`.
- Every function: bounded timeout, returns `None` on any failure, never
  raises into the caller.

### Unified event log

`~/.ciel/system1/events.jsonl` replaces `shadow.log`. Every record:
`{ts, surface, state_ref, question, answer, confidence, latency_ms, model}`,
where `state_ref` is the activity.log timestamp (correlation, not
duplication). 4 MiB rotation as today.

### Surfaces — each behind `CIEL_SYSTEM1_SURFACES`, each graduates alone

| Surface | Primitive | Trigger | Graduation path |
| --- | --- | --- | --- |
| `pre_tool_risk` (live) | choice safe/dangerous | every tool call | shadow → advisory-confirm on `regex=allow ∧ dangerous@high-conf` |
| `router` | choice over top-k candidates | ambiguous trigger-registry match | shadow → fast-path when top-1 margin > τ |
| `council_prescreen` | choice routine/escalate | council-scope events | shadow → skip convening on `routine@high-conf` (saves ~10 model calls) |
| `completion_check` | score 1–5 rubric | post-task verify | shadow → augment `paired_eval.py` scoring |
| `trust_anomaly` | choice expected/anomalous | skill promotion + periodic audit | shadow → flag for human review |
| `memory_salience` | choice store/skip | post-tool events | shadow → filter memory ingestion |

`choice` with two neutral options is used in place of `noul` — the base
checkpoint's `noul` head follows label wording rather than state.

### Phases

- **P1 — refactor + calibration harness.** `system1.py`, unified log, cache,
  concurrency bound. `scripts/system1_eval.py` scores the endpoint against
  the 41-case red-team corpus plus harvested shadow traffic; emits
  `risk/system1_calibration.json` (precision/recall/ECE/latency per surface).
- **P2 — shadow expansion.** `router` + `council_prescreen` join shadow mode.
- **P3 — advisory promotion.** Per-surface promotion, each requiring a
  calibration artifact AND a council vote. Advisory = confirm-prompt or
  annotation; never a deny.
- **P4 — fine-tune.** RLCD `ciel-risk` checkpoint on the accumulated corpus;
  re-calibrate; the model card's near-chance zero-shot baseline is the floor
  this phase exists to beat.
- **P5 — enforcement consideration.** Only `escalate-to-confirm` is ever on
  the table. Hard rules remain deterministic regex forever.

## Invariants (non-negotiable)

1. Model output never denies, never weakens a hard rule, never overrides a
   regex deny.
2. Fail-open everywhere; `CIEL_SYSTEM1_DISABLED=1` is the global kill.
3. Bounded concurrency + cache before any surface expansion.
4. Key stays in `system1/env` (gitignored); localhost-only binding.
5. No surface graduates on intuition — a committed calibration artifact is
   the evidence.

## Rejected alternatives

- **Synchronous shadow in the hook path**: measured 4.3s/call on CPU —
  unacceptable in a PreToolUse gate.
- **`noul` for pre-screening**: label-following bug in base checkpoints;
  two-option `choice` with neutral keys is the documented workaround.
- **One calibration for all surfaces**: each surface's distribution differs;
  per-surface artifacts prevent one good surface laundering a bad one.
- **Unbounded detached spawn**: current `Popen`-per-call survives shadow-only
  volume but would fork-storm under expansion; bounded before expansion.

## Consequences

- `risk_policy.py` slims back to policy evaluation; the client moves to
  `system1.py` with `risk_policy` as its first consumer.
- `shadow.log` readers migrate to `events.jsonl` (log format gains
  `surface`; the field is additive).
- The calibration artifact becomes a new committed file under `risk/` —
  evidence trail like `skill_scan_baseline.json`.
