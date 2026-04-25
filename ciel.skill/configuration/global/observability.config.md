# observability.config — Global Observability

```yaml

# <anchor:start>

observability:
  log_verbosity: info               # info|debug|trace
  log_rotate_hour: 0
  log_retention_days: 90
  redact_secrets: true              # Constitutional: locked true
  session_summary: on_error         # off|on|on_error
  session_summary_retention_days: 30
  otel:
    enabled: false
    endpoint: null
    service_name: ciel
    resource_attrs:
      deployment: dev
    sampling:
      default: 0.1
      critical: 1.0
      council: 1.0

# <anchor:end>

```

## Notes

- `redact_secrets: true` is Constitutional. Unset → fail-closed (no writes at all).
- Debug mode activation: `CIEL_DEBUG=1` env or `/ciel --debug`.
- Session summary written to `~/.ciel/sessions/` and optionally emitted to user.

See `observability/OBSERVABILITY.md`.
