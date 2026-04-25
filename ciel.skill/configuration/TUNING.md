# TUNING — Self-Tuning Protocol

How Ciel proposes and applies config changes.

## Categories

| Category | Examples | Gate |
| --- | --- | --- |
| trivial | cache TTL ±10%, fast_path_floor ±0.02 | auto |
| standard | weight shifts, thresholds mid-range | Council |
| structural | backend swap, adapter enable/disable | Council + user |
| constitutional | weights.safety below 0.20, isolation_strict=false, redact_secrets=false | **rejected** |

## Auto-Apply Range

For `trivial`, Ciel applies within `± auto_tune_range` (default 10% or equivalent) without Council. Any change outside that range → standard.

## Proposal Flow

1. Trigger identifies suboptimal setting (e.g. low cache hit rate → propose higher TTL).
2. Generate proposal with evidence (before/after metrics, projection).
3. Classify category.
4. Gate.
5. Apply via `configuration/global/*.config.md` edit (anchor-updated).
6. Post-change: observe per `self_improvement/REGRESSION_DETECTION.md`. Rollback on regression.

## User Override

User-set values pin. Ciel does not auto-tune a user-pinned field without explicit reset. Annotation `# pinned` next to the value preserves it across sweeps.

## Config File Shape

All `configuration/global/*.config.md` share:

```yaml

# File: <name>.config.md — managed by Ciel.

<anchor:start>
<YAML body>
<anchor:end>
```

Anchors are visible comments so users can edit safely between them.

## History

Every config change is a git commit (`config_tune: <field>: <old> -> <new>`) with:

- trigger that motivated the change,
- evidence snapshot,
- council run id if gated.

## Disabling Auto-Tune

```yaml
improvement:
  auto_tune: false
```

All tuning then requires manual user action. Default is `true`.
