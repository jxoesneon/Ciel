---
name: council_runner
version: 1.0.0
description: Run Council of Five deliberations — stage orchestration, result aggregation.
triggers: [council, vote, deliberate, review artifact]
tags: [council, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: partial }
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
- Generic: sequential inline.
