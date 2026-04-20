# RISK — Model Master

Every operation is classified. Classification drives gating.

## Levels

| Level | Authority | Gate |
| --- | --- | --- |
| low | autonomous | log only |
| mid | autonomous with judge | LLM judge before proceeding |
| high | council-gated | full Council invocation |
| critical | user escalation | user must approve, even after Council |

## Files

- `CLASSIFICATION.md` — criteria and examples per level.
- `LOW_RISK.md` — low-risk policy.
- `MID_HIGH_RISK.md` — mid/high policy + LLM judge flow.
- `CRITICAL_RISK.md` — critical policy (locked).
- `LLM_JUDGE.md` — judge protocol.
- `ESCALATION_LADDER.md` — end-to-end decision flow.

## Cross-Cuts

Risk classification is consulted in:

- `router/ROUTER.md` pre-execution.
- `adapters/*/HOOKS.md` pre-flight hooks.
- `acquisition/ACQUISITION.md` for tier-3 skills.
- `self_improvement/` before applying proposals.

## Configurability

- `risk.config.classification_weights` — tweak axis weights within Constitutional caps.
- `risk.config.mid_threshold`, `high_threshold`, `critical_threshold` — numeric boundaries.
- `escalation_override` in `configuration/local/escalation.config.md` shifts the `autonomous vs council vs escalate` mapping based on project profile (research/development/production/regulated).

Constitutional floor: critical-level operations always escalate.
