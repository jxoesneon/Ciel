# TRUST_MODEL

Untrusted → sandboxed → validated → promoted.

## States

| State | Description |
| --- | --- |
| `untrusted` | just arrived, harmonized, never executed |
| `sandboxed` | executed in isolation with synthetic inputs; trace captured |
| `validated` | Council-approved; registered; under observation |
| `promoted` | matured: confidence high, used repeatedly, can be a composition component |
| `suspect` | observed anomalies; under review by self-improvement loop |
| `deprecated` | scheduled for removal |

Transitions are logged; reverse transitions require Council.

## Static scan (mandatory)

Every new skill is scanned with `skillfrisk` (`scripts/scan_skills.py`) while `untrusted`, before any sandboxed execution. A skill whose scan reports unbaselined findings cannot leave `untrusted` until the findings are remediated, or individually reviewed and recorded with a justification in `ciel.skill/risk/skill_scan_baseline.json`. The scan report is attached to the trust record and re-checked on every skill update.

## Paired evaluation (mandatory before `validated`)

Any skill mutation or `sandboxed` → `validated` promotion must carry a paired-eval evidence report from `scripts/paired_eval.py`: the eval task set is run with and without the candidate skill in isolated workspaces, and each arm is scored by the task's own `verify.sh`. The gate fails on any `regression` outcome (control pass → treatment fail); `--require-improvement` additionally demands at least one `improvement`. The JSON report is attached to the Council docket — self-evaluation prose is not evidence. See `ciel.skill/self_improvement/PAIRED_EVAL.md`.

## Trust Score

```text
trust = 0.4 * origin_tier_bias

      + 0.2 * sandbox_pass_rate
      + 0.2 * production_success_rate
      + 0.1 * council_pass_count
      + 0.1 * age_bonus (log-scaled, capped)

```

Origin tier bias: Tier 1 = 1.0, Tier 2 = 0.8, Tier 3 = 0.4, Composition = average of components.

## Use in Routing

Router's confidence uses trust as a multiplier on the fast-path and reasoning scores. Low-trust skills require higher pattern match before routing.

## Demotion

If production_success_rate drops or Safety flags accumulate:

- `validated` → `suspect`, Council review triggered.
- `suspect` without remediation → `deprecated`.
- `deprecated` → removal after sweep interval.

## Audit Trail

Every state change is a git commit + activity.log entry + MemPalace event.
