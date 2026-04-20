# OVERRIDE — Manual Escalation Override

User may override the auto-calibrated escalation threshold per project.

## File

`<project>/.ciel/escalation.json`:

```json
{
  "auto_detected": "development",
  "override": "production",
  "effective": "production",
  "override_reason": "deploys to staging from feature branches",
  "override_set_by": "user|ciel-proposal",
  "override_set_at": "..."
}
```

`effective = override ?? auto_detected`.

## Setting the Override

- `/ciel override set <research|development|production|regulated>` — persists to `escalation.json`.
- Editing `escalation.json` directly is respected; Ciel reloads on next operation.
- Via `configuration/local/escalation.config.md`:

```yaml
escalation:
  override: production
  reason: "staging deploy pipeline"
```

## Constitutional Floor

Constitutional invariant (see `core/CONSTITUTION.md`):

- Override **cannot** set a value more permissive than the auto-detected category by more than one step.
- Override **cannot** raise permissiveness past `research`.
- Override **can always tighten** (regulated > production > development > research).

Violations are rejected with a message explaining the floor.

## Proposal by Ciel

Ciel may propose an override after observing recurrent context change:

- if detection signals drifted, she may suggest a new auto-detection value, or
- suggest a manual override if user behaviour implies a different category (e.g. user repeatedly opts for stricter gating).

All proposals are queued; user confirms.
