# CALIBRATION — Escalation Threshold

Based on detected context, choose an initial escalation threshold for this project.

## Threshold Scale

| Threshold | Description | Typical use |
| --- | --- | --- |
| `research` | Most permissive — Ciel acts liberally; only critical risk escalates | exploratory notebooks, personal scratch |
| `development` | Default — mid/high risk Council-gated, critical escalates | standard dev work |
| `production` | Conservative — mid risk Council-gated; high+ always escalates | production repo |
| `regulated` | Most restrictive — all mid+ escalates; only low acts autonomously | healthcare, finance, infra |

## Heuristics

- Repo has `production`, `main`, `release` branches + CI deploy to a public target → `production`.
- Repo has compliance-adjacent files (`SECURITY.md`, `COMPLIANCE.md`, mentions of HIPAA/SOX/PCI) → `regulated`.
- Repo has no CI, small scale, personal author → `research`.
- Otherwise → `development`.

## Signals (additive)

| Signal | Shift |
| --- | --- |
| Secrets in repo history | +1 toward regulated |
| `.env.example` present | baseline |
| `docker-compose` with databases | +1 toward production |
| Active PRs > 10 / week | +1 toward production |

## Output

Written to `<project>/.ciel/escalation.json`:

```json
{
  "auto_detected": "development",
  "override": null,
  "effective": "development",
  "rationale": "detected CI + tests + active maintenance; standard dev profile",
  "calibrated_at": "..."
}
```

## Override

User can override via `OVERRIDE.md` → `configuration/local/escalation.config.md`. Constitutional floor is `research` (cannot go below — Ciel will not disable her Council for self-mutation even if user requests).

## Re-Calibration

Context can change. Ciel re-calibrates:

- on manual `/ciel-init`,
- on major context signal changes (added CI, added compliance file),
- on user request `/ciel-calibrate`.
