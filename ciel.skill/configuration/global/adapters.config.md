# adapters.config — Global Adapters

```yaml

# <anchor:start>

adapters:
  claude_code:
    hooks:
      preflight: true
      postflight: true
      failure: true
      permission: true
    computer_use: preview           # false|preview|true
    ultraplan: true
    prompt_cache: true
    remote_control:
      enabled: false
      notification_channels: [push]
      quiet_hours: null
  gemini_cli:
    hooks:
      preinvoke: true
      postinvoke: true
      error: true
      permission: true
      session: true
    plan_mode: true
    checkpointing: true
    a2a_remote_subagents: false      # user opt-in
    multimodal: true
    model_routing:
      budget_per_session_usd: 3.00
      alert_on: 0.8
      hard_stop_on: 1.0
      fallback_confidence_bump: 0.05
  generic:
    research_enabled: true

# <anchor:end>

```

## Notes

- Disabling a hook degrades the corresponding Ciel capability with a logged notice.
- `computer_use` / `a2a_remote_subagents` are user opt-ins due to blast radius.
- Model routing budgets apply per-session; exceedance escalates.

See per-adapter files in `adapters/<runtime>/` for details.
