# SCHEMA — Config Reference

Every Ciel config field. Canonical.

## Top Level

```yaml
version: 1                              # schema version
runtime_prefs:
  preferred: auto                       # auto | claude-code | gemini-cli | generic
  fallback_order: [claude-code, gemini-cli, generic]
telemetry:
  otel_enabled: false
  otel_endpoint: null
```

## Router

```yaml
router:
  fast_path_floor: 0.80           # 0..1
  reasoning_floor: 0.70           # 0..1
  cache_ttl_minutes: 60
  context_budget:
    total_max_tokens: 32000
    registry_l0_max: 4000
    candidate_l1_k: 5
    candidate_l1_max: 3000
    council_stage1_max: 8000
    acquisition_l2_max: 16000
  prompt_cache:
    floor: 0.50
  plan_mode:
    budget_tokens: 8000
```

## Council

```yaml
council:
  pass_score: 6
  weighted_pass: 6.5
  reject_threshold: 4.5
  majority_required: 3
  weights:
    coherence: 0.20
    capability: 0.20
    safety: 0.25                  # Constitutional floor: 0.20
    efficiency: 0.15
    evolution: 0.20
  anonymize_stage2: true           # Constitutional floor: true
  stage_timeout_s: 60
  local_quorum_min: 3
```

## Acquisition

```yaml
acquisition:
  tier1_timeout_s: 10
  tier2_timeout_s: 30
  tier3_timeout_s: 120
  total_wall_budget_s: 300
  token_budget: 80000
  cve_threshold: high              # skip sources w/ deps above this CVSS
  sandbox_limits:
    cpu_pct: 50
    memory_mb: 512
    wall_s: 60
  tier1_floor: 0.5
  sandbox_retention_hours: 48
```

## Memory

```yaml
memory:
  backend: mempalace                # mempalace|sqlite|filesystem|custom
  auto_update: true
  version_pin: null
  reinstall_check_days: 7
  health_check_interval_minutes: 60
  isolation_strict: true            # Constitutional floor: true
  partition_size_limit_mb: 1024
  fallback_snapshot_retention_days: 30
```

## Risk

```yaml
risk:
  mid_threshold: 3.0
  high_threshold: 6.0
  critical_threshold: 8.5
  judge_confidence_floor: 0.70
  mid_judge_model: auto
  mid_cost_usd_threshold: 0.10
  high_cost_usd_threshold: 1.00
  cost_threshold: 0.20
  plan_mode_gate: false
  classification_weights:
    reversibility: 0.25
    blast_radius: 0.20
    external_impact: 0.20
    data_sensitivity: 0.15
    cost: 0.10
    novelty: 0.10
```

## Improvement

```yaml
improvement:
  global_max_per_day: 20
  local_max_per_day: 10
  trigger_dedup_window_hours: 6
  suppression_days: 7
  sweep_interval: weekly             # daily|weekly|monthly
  checkpoint_stale_hours: 72
  regression:
    watch_invocations: 20
    watch_hours: 48
```

## Observability

```yaml
observability:
  log_verbosity: info                # info|debug|trace
  log_rotate_hour: 0
  log_retention_days: 90
  redact_secrets: true               # Constitutional: locked true
  session_summary: on_error          # off|on|on_error
  session_summary_retention_days: 30
  otel:
    enabled: false
    endpoint: null
    service_name: ciel
    sampling:
      default: 0.1
      critical: 1.0
      council: 1.0
```

## Adapters

```yaml
adapters:
  claude_code:
    hooks: { preflight: true, postflight: true, failure: true, permission: true }
    computer_use: preview            # false|preview|true
    remote_control:
      enabled: false
      notification_channels: [push]
      quiet_hours: null
  gemini_cli:
    hooks: { preinvoke: true, postinvoke: true, error: true, permission: true, session: true }
    model_routing:
      budget_per_session_usd: 3.00
      alert_on: 0.8
      hard_stop_on: 1.0
      fallback_confidence_bump: 0.05
  generic:
    research_enabled: true
```

## Project (local only)

```yaml
project:
  rules: [...]                        # see local/rules.config.md
  overrides:                          # see local/overrides.config.md
  escalation:
    override: null                    # see local/escalation.config.md
```
