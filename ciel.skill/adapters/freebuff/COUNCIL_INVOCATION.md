# COUNCIL_INVOCATION — Freebuff

Freebuff has **no native subagent dispatch** and **no parallel execution
tracks**. The Council of Five therefore runs as **sequential in-context
deliberation** per stage, with the Chairman in the same session. This is the
degraded `mode: "inline"` path defined in the contract — it is correct, but
slower, and it must always be *declared*, never presented as isolated
dispatch.

## Topology

```text
single session (Ciel = Chairman + each member in turn)
  │
  ├── Council/Safety       (deliberated first — Safety holds veto authority)
  ├── Council/Capability
  ├── Council/Coherence
  ├── Council/Efficiency
  └── Council/Evolution
  │
  └── Chairman synthesizes (same session, all five verdicts)
```

Each member is deliberated as an **individual persona** — loaded from
`council/members/<NAME>.md` via the `skill`/file-read mechanism, with its own
rubrics (`council/rubrics/SCORING.md`, `VETO_CONDITIONS.md`). Never merge
members into a single "the council thinks…" pass, never truncate a member's
verdict. The Chairman may not adopt a subset.

## Stage Flow

1. **Evidence pack**: Chairman writes the evidence pack to a readable path
   (e.g. `~/.ciel/council/<run_id>/evidence.md`) so every member reviews the
   identical artifact.
2. **Stage 1**: for each member in turn, deliberate in its persona and emit
   strict JSON `{member, stage:1, score, rationale, mechanism_verdicts?,
   flags, requests}`. Write each verdict to
   `members/<member>.stage1.json` as it is produced (crash-safe checkpointing
   — a long sequential run must not lose completed verdicts).
3. **Cross-review**: Chairman anonymizes Stage 1 outputs (peers labelled A–D,
   identities withheld) and re-deliberates each member against the anonymized
   set → `members/<member>.stage2.json`.
4. **Stage 3**: Chairman synthesizes per
   `prompts/council/chairman_synthesis.md` and writes `verdict.json`.

## Enforcement

Member-as-subagent is a **mechanism, not prose** — on this runtime the
mechanism is sequential inline deliberation, and the receipts must say so.
Every run must produce:

```text
~/.ciel/council/<run_id>/
├── spawn_receipts.json   # {"mode":"inline","runtime":"freebuff","members":{name:{stage1:true,stage2:true}}}
├── evidence.md
├── members/<member>.stage1.json
├── members/<member>.stage2.json
└── verdict.json          # chairman docket
```

- `spawn_receipts.json` records `"mode": "inline"` — the honest receipt.
  `mode: "subagent"` MUST NOT be written on this runtime; there is no
  dispatch primitive to back it.
- After the docket is written, run
  `python3 scripts/council_verify.py <run_id>` — it validates completeness
  and reports the run as `unverified_member_isolation` (expected here). Its
  structured verdicts go to
  `~/.ciel/improvements/signals/council-<run_id>.json` for the improvement
  loop.
- If a future Freebuff session exposes an agent-spawning tool, re-run the
  Council with individual dispatches, set `"mode": "subagent"`, and upgrade
  `adapters/freebuff/ADAPTER.md` capability flags.

## Failure

- There are no subagent timeouts; a member whose deliberation produces
  malformed JSON is re-deliberated once, then recorded as abstention with the
  parse error attached.
- Safety abstention → re-deliberate once; second abstention escalates to the
  user via `ask_user` before any pass verdict.
- Context pressure: five members × two stages inline is token-heavy. If the
  session budget will not hold full two-stage deliberation, drop to
  Stage 1 + Chairman synthesis only, record the elision in
  `spawn_receipts.json` (`"stages": {"cross_review": false}`), and note it in
  the docket. Never fabricate stage-2 verdicts.
