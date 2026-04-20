# ciel.config — Top-Level Global

```yaml
# <anchor:start>
version: 1
runtime_prefs:
  preferred: auto
  fallback_order: [claude-code, gemini-cli, generic]
telemetry:
  otel_enabled: false
  otel_endpoint: null
backup:
  cadence: daily
  retention_count: 14
  retention_days: 30
  target: ~/.ciel/backups/
  remote: null
improvement:
  auto_tune: true
# <anchor:end>
```

Primary knobs:

- `runtime_prefs.preferred` — hard-pin to `claude-code` or `gemini-cli` if desired.
- `telemetry.otel_*` — enable and endpoint for external observability.
- `backup.*` — global backup cadence and retention.
- `improvement.auto_tune` — whether Ciel may self-tune within trivial range.

See `configuration/SCHEMA.md` for full field reference.
