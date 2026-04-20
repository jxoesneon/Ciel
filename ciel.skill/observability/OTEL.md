# OTEL — OpenTelemetry Integration

Ciel emits OTLP traces / metrics when configured, natively under Claude Code and via adapter under Gemini CLI.

## Configuration

```yaml
observability:
  otel:
    enabled: false
    endpoint: null            # e.g., https://otel-collector.example.com:4318
    service_name: ciel
    resource_attrs:
      deployment: dev
    sampling:
      default: 0.1
      critical: 1.0           # always sample critical risk
      council: 1.0
```

## Traces

Emitted spans (selected):

| Span | Parent | Attributes |
| --- | --- | --- |
| `ciel.route` | session | `risk`, `path`, `skill`, `runtime` |
| `ciel.fast_path` | `ciel.route` | `confidence`, `candidates` |
| `ciel.reasoning_path` | `ciel.route` | `confidence`, `steps`, `gaps` |
| `ciel.acquisition` | `ciel.route` | `tier`, `source`, `council_run` |
| `ciel.council` | `ciel.route`/`ciel.acquisition` | `scope`, `verdict`, `weighted_score` |
| `ciel.council.member` | `ciel.council` | `lens`, `stage`, `score` |
| `ciel.llm_judge` | hook | `decision`, `confidence`, `model` |
| `ciel.memory.op` | any | `partition`, `op`, `keys` |
| `ciel.escalation` | session | `reason`, `risk`, `resolution` |

## Metrics

- `ciel.requests.count{path, runtime, outcome}`
- `ciel.route.latency_ms{path}`
- `ciel.skill.success_rate{skill_id}`
- `ciel.acquisition.hit_rate{tier}`
- `ciel.council.verdict{scope, verdict}`
- `ciel.cost.usd_total{runtime, model}`
- `ciel.cache.hit_ratio{segment}`

## Claude Code Integration

Claude Code supports OTEL natively. Ciel's spans nest inside Claude Code's own session span via context propagation.

## Gemini CLI Integration

Gemini CLI's telemetry diff mechanism is consumed; Ciel translates it into OTEL spans where possible. Where not possible, Ciel emits her own spans independently.

## Privacy

- Attribute values are redacted using the same rules as `ACTIVITY_LOG.md`.
- No request/response bodies are attached to spans beyond hashed identifiers.
