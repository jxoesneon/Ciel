# CONFIG: Rate Limits and Resource Awareness

This file governs Ciel's multi-agent fan-out parameters to prevent API exhaustion (HTTP 429).

```yaml
resource_awareness:
  enabled: true
  max_concurrent_subagents_pro: 3
  max_concurrent_subagents_flash: 5
  fallback_strategy:
    - action: "throttle_and_wait"
      duration_seconds: 10
    - action: "degrade_model"
      target: "flash"
    - action: "script_override"
      condition: "massive_file_count > 20"
      preferred_tool: "run_command (python/bash)"
```

## Directives
1. **Never** spawn a subagent per-file for repository-wide refactoring. Use single-shot automation scripts instead.
2. **Double Agentic Loops** must stagger their inner loop invocations sequentially if the outer loop size exceeds 2.
3. **Monitor Subagent State:** On `errored` states matching `RESOURCE_EXHAUSTED`, instantly trigger the fallback strategy.
