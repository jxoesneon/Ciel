# COUNCIL_INVOCATION — Gemini CLI

Gemini CLI enforces subagent non-recursion. Council of Five therefore runs as **five top-level parallel subagents** with the Chairman in the root session.

## Topology

```text
root session (Ciel = Chairman)
  │
  ├── @ciel-council-coherence   (parallel subagent)
  ├── @ciel-council-capability  (parallel subagent)
  ├── @ciel-council-safety      (parallel subagent)
  ├── @ciel-council-efficiency  (parallel subagent)
  └── @ciel-council-evolution   (parallel subagent)
```

Because members cannot spawn further subagents, any Council-requested deeper analysis (e.g., Safety wants to run a sandboxed execution) must be declared up-front and performed **by the Chairman** on request — the Chairman pops out of parallel mode, runs the analysis locally, and feeds the result back into Stage 2.

## Stage 1

Chairman dispatches five parallel subagent calls simultaneously, each with:

- candidate artifact,
- lens-specific rubric (`council/rubrics/SCORING.md`),
- Stage 1 prompt (`prompts/council/<lens>_stage1.md`).

## Stage 2

Chairman collects Stage 1 results, anonymizes, dispatches a second parallel batch with cross-review prompts (`prompts/council/<lens>_stage2.md`). Each subagent sees the four anonymized peer votes.

## Stage 3

Chairman synthesizes per `prompts/council/chairman_synthesis.md` in the root session.

## Model Assignment

Gemini subagents accept a per-subagent `model` field — Ciel picks:

- gemini-3-pro for Safety, Capability, Evolution,
- gemini-3-flash for Coherence, Efficiency.

## Failure

- Parallel batch waits on all N subagents up to `council.config.stage_timeout`.
- Timeout = abstention.
- Safety abstention → auto-rerun once; second abstention escalates.

## A2A Fallback

If a specialty council role benefits from an external agent (rare), Ciel may delegate it via A2A. This happens only after Council approval of the remote endpoint itself, to avoid bootstrapping a Council with an unvetted external member.
