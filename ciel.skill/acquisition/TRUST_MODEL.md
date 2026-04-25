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
