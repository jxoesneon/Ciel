---
name: council_runner
version: 1.0.0
description: Run Council of Five deliberations — stage orchestration, result aggregation.
triggers: [council, vote, deliberate, review artifact]
tags: [council, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [mempalace_manager/SKILL.md, markdown_processor/SKILL.md] }
---

# council_runner

Orchestrate the 3-stage Council of Five.

## Operations

- `council.run(scope, artifact, context?)` — returns Chairman verdict.
- `council.stage1(members, artifact)` — parallel scoring.
- `council.anonymize(stage1_outputs)` — Stage 2 prep.
- `council.stage2(members, anonymized)` — cross-review.
- `council.chairman(stage2_outputs, weights)` — synthesis.
- `council.record(run_id, payload)` — persist to `~/.ciel/council/<id>/`.

## I/O Contract

```yaml
io_contract:
  input: { scope, artifact, "context?" }
  output: { verdict, weighted_score, pivotal_lens, votes, run_id }
  idempotent: no (but deterministic with seed)
  side_effects: [fs, "network?"]
```

## Runtime Adaptation

- Claude Code: nested subagents (see `adapters/claude_code/COUNCIL_INVOCATION.md`).
- Gemini CLI: parallel top-level subagents.
- Devin: five parallel `run_subagent` dispatches per stage (see `adapters/devin/COUNCIL_INVOCATION.md`).
- Generic: sequential inline.

## Enforcement Contract

Member-as-subagent is a mechanism, not prose (docket
`council-20260923-conversation-audit`, mechanism M1). Every run MUST:

1. Write the evidence pack to a readable path and dispatch each member as
   an individual subagent — never inline deliberation, never truncated
   member output. The Chairman synthesizes ALL member verdicts; it may not
   adopt a subset.
2. Record per-member spawn receipts and stage artifacts under
   `~/.ciel/council/<run_id>/`:

   ```text
   spawn_receipts.json   {"mode":"subagent"|"inline","members":{name:{stage1_agent_id,stage2_agent_id}}}
   members/<member>.stage1.json
   members/<member>.stage2.json
   verdict.json          # chairman docket
   ```

3. `mode: "inline"` is a degraded fallback only — it MUST be declared;
   verification flags it `unverified_member_isolation`. A run with no
   receipts fails verification outright.
4. After the docket, run `python3 scripts/council_verify.py <run_id>` —
   validates completeness and emits structured member verdicts to
   `~/.ciel/improvements/signals/council-<run_id>.json` for the
   improvement loop.
