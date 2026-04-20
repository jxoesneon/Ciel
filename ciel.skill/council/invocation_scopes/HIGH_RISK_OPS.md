# SCOPE — HIGH_RISK_OPS

Triage: irreversible / destructive operations the user (or a skill) is asking Ciel to perform.

## Trigger

`risk/CLASSIFICATION.md` returns `high` or above for the operation.

Examples:

- `git push --force` to a shared branch.
- `npm publish` / `cargo publish`.
- mass file deletions.
- `DROP TABLE` / `DROP DATABASE` on non-local DBs.
- sending email / SMS / paid API call to external parties.
- modifying OS-level state (`systemctl`, kernel modules).
- rewriting git history on a shared remote.

## Preamble

> You are evaluating a **high-risk operation** before execution. Operation: `<op>`. Classification: `<high|critical>`. Reversibility: `<reversible|irreversible>`. Apply your lens.

## Lens Emphasis

- **Safety** — primary. Evaluates blast radius, rollback availability, explicit consent evidence.
- **Capability** — is the operation necessary, or is there a safer alternative?
- **Efficiency** — not primary, but flags if a lighter op achieves the same result.
- **Coherence** — ensures alignment with project rules (see `configuration/local/rules.config.md`).
- **Evolution** — evaluates whether proceeding teaches a reusable pattern.

## Thresholds

```yaml
high_risk:
  pass_weighted_score: 7.0
  safety_min: 6
  require_explicit_user_consent: true   # for irreversible ops
```

## Critical-Risk Handling

If `risk/CLASSIFICATION.md` returns `critical`, Council is still invoked for the record, **but** user escalation is mandatory regardless of Council pass (see `risk/CRITICAL_RISK.md`). Council can strengthen the case for the user prompt with its analysis.

## Post-Execution

High-risk ops automatically trigger `self_improvement/OUTCOME_SCORING.md` with elevated weight. Post-mortem is written to `~/.ciel/high_risk/<run_id>.md`.
