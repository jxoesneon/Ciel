# ESCALATION

When the Council cannot reach a clean verdict, what happens next.

## Deadlock Cases

| Case | Handling |
| --- | --- |
| 2-2-1 (abstention) | Chairman re-dispatches the abstaining member once. If still abstaining → meta-judgment. |
| Scores clustered around threshold ± 1.0 | Chairman meta-judgment (`CHAIRMAN.md`) |
| Meta-judgment cannot decide | Escalate to user |
| Safety veto + Evolution strong-pass | Auto-reject; log the contradiction as a signal for self-improvement |
| Stage 1 timeout ≥ 3 members | Rerun Stage 1 once; if still failing → escalate |

## User Escalation Format

When Chairman escalates:

```text
Council deadlock on <artifact>.

Votes:

- Coherence:  7/10 — ...
- Capability: 6/10 — ...
- Safety:     6/10 — ...
- Efficiency: 5/10 — ...
- Evolution:  7/10 — ...

Chairman meta-judgment: unable to resolve without more evidence.
Pivotal lens: Safety (close to veto threshold on <concern>).

Proposed defaults:
  A) proceed with the mitigation listed above
  B) reject and try a different approach
  C) escalate further research before deciding

Waiting for decision.
```

## Post-Escalation

- User decision is recorded in `~/.ciel/council/<run_id>/user_decision.json` and counts as a signal for improvement (pattern: when does user override Council?).
- No repeated escalation without new evidence: if the identical artifact recurs, Ciel surfaces the previous decision and does not re-run Council unless explicitly asked.

## Never Bypass

Escalation never allows a Safety veto to be overridden. It only resolves deadlocks, deadlock-adjacent scoring, or abstention deadlocks. Safety-veto rejections require the full Constitutional amendment procedure (`core/CONSTITUTION.md`).
