# COUNCIL_INVOCATION — Devin CLI

Devin supports up to 5 concurrent background subagents via `run_subagent`.
Council of Five therefore runs as **five parallel `run_subagent` dispatches**
per stage, with the Chairman in the root session.

## Topology

```text
root session (Ciel = Chairman)
  │
  ├── run_subagent: Council/Coherence   (stage 1, then stage 2)
  ├── run_subagent: Council/Capability
  ├── run_subagent: Council/Safety
  ├── run_subagent: Council/Efficiency
  └── run_subagent: Council/Evolution
```

Each member is an **individual subagent** — never inline deliberation, never
truncated member output. The Chairman synthesizes all five verdicts; it may
not adopt a subset.

## Stage Flow

1. **Dispatch**: Chairman writes the evidence pack to a readable path, then
   issues five parallel `run_subagent` calls — each scoped to its member
   persona (`council/members/<NAME>.md`), rubrics
   (`council/rubrics/SCORING.md`, `VETO_CONDITIONS.md`), and the evidence
   pack. Read-only (`subagent_explore`) unless write access is required.
2. **Stage 1**: each member returns strict JSON `{member, stage:1, score,
   rationale, mechanism_verdicts?, flags, requests}`.
3. **Cross-review**: Chairman anonymizes Stage 1 outputs (peers labelled
   A–D, identities withheld) and dispatches a second parallel batch via
   `run_subagent` (fresh subagents or `resume`).
4. **Stage 3**: Chairman synthesizes in the root session per
   `prompts/council/chairman_synthesis.md` and writes the docket.

## Enforcement (council-20260923-conversation-audit, M1)

Member-as-subagent is a **mechanism, not prose**. Every run must produce:

```text
~/.ciel/council/<run_id>/
├── spawn_receipts.json   # {"mode":"subagent"|"inline", "members":{name:{stage1_agent_id,stage2_agent_id}}}
├── members/<member>.stage1.json
├── members/<member>.stage2.json
└── verdict.json          # chairman docket
```

- `spawn_receipts.json` records the subagent `agent_id`s per member per
  stage — the receipt proving individual dispatch.
- `mode: "inline"` is permitted as a degraded fallback but **must** be
  declared; verification reports it as `unverified_member_isolation` —
  never silently satisfy the contract.
- After the docket is written, run
  `python3 scripts/council_verify.py <run_id>` — it validates completeness
  and emits the run's structured verdicts to
  `~/.ciel/improvements/signals/council-<run_id>.json` for the improvement
  loop.

## Failure

- Parallel batch waits on all 5 subagents; a timeout = member abstention.
- Safety abstention → re-dispatch once; second abstention escalates to the
  user before any pass verdict.
- `run_subagent` concurrency limit is 5 — the Council fits exactly; queue
  any additional dispatch until a slot frees.
