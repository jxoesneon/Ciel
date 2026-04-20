# COUNCIL_INVOCATION — Claude Code

Claude Code allows nested subagent trees. Ciel's Council of Five runs as five parallel subagents, coordinated by a parent Chairman context (Ciel herself).

## Topology

```text
Ciel (chairman)
├── council-coherence (subagent, isolated context)
├── council-capability (subagent, isolated context)
├── council-safety    (subagent, isolated context)
├── council-efficiency (subagent, isolated context)
└── council-evolution (subagent, isolated context)
```

Each subagent file lives under `.claude/agents/ciel-council-<lens>.md` (installed at init).

## Stage Flow

**Stage 1** — Chairman fires five `Agent` tool calls in parallel, each receiving:

- candidate artifact (skill bundle or proposal diff),
- lens-specific rubric from `council/rubrics/SCORING.md`,
- lens-specific prompt from `prompts/council/<lens>_stage1.md`.

Each returns a structured vote: `{ score, rationale, flags }`.

**Stage 2** — Chairman anonymizes the five Stage 1 outputs, fires five *new* Agent calls with each member seeing the four anonymized peer outputs (see `council/ANONYMIZATION.md`). Each returns a revised vote.

**Stage 3** — Chairman synthesizes per `prompts/council/chairman_synthesis.md` and `council/CHAIRMAN.md`.

## Context Budgets

Each member receives L1 skill content by default (`router/CONTEXT_BUDGET.md`). A member requesting L2 must justify via a preamble in their Stage 1 output; the Chairman re-dispatches that single member with L2 if justified.

## Cost

Two rounds of five parallel calls = 10 subagent invocations per Council run. Amortized by prompt caching (rubrics + personas are cached; only the candidate artifact is cache-cold).

## Failure

- Any Stage 1 subagent timeout → Chairman treats that member's vote as an abstention; requires ≥ 3 non-abstentions for quorum.
- Safety subagent failure → Chairman re-runs it once; second failure escalates to user.
