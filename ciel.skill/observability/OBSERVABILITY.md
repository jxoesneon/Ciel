# OBSERVABILITY — Master

What Ciel surfaces, when, and how.

## Layers

- **Native host surfaces** — the agent LLM's own thoughts/summaries appear in the user's terminal.
- **Activity log** — `ACTIVITY_LOG.md`, append-only, structured.
- **OpenTelemetry** — `OTEL.md` (optional).
- **Session summaries** — `SESSION_SUMMARY.md`, per session, in user-visible format.
- **Debug mode** — `DEBUG.md`, verbose traces.

## Principles

- **No silent action.** Every operation appears in at least one observable surface.
- **Structured > prose** in logs; agent-readable for self-improvement loop.
- **Prose > structured** in user summaries; human-readable.
- **Opt-in verbosity.** Default is concise; debug mode or `/ciel-status` opens the firehose.
- **Privacy-aware.** Secrets are redacted at capture time, not at display time.

## Surfaces

| Audience | Surface | Cadence |
| --- | --- | --- |
| User | Agent's native summary | per message |
| User | `/ciel-status` | on demand |
| User | Escalation prompts | on demand |
| User | Post-session summary | at session end (if configured) |
| Ciel | `activity.log` | every op |
| Ciel | MemPalace traces | every op |
| External (OTEL) | Telemetry endpoint | configured |

## Config

See `configuration/global/observability.config.md`:

- `log_verbosity` — info / debug / trace
- `otel_endpoint` — null or URL
- `redact_secrets` — true (Constitutional — cannot disable)
- `session_summary` — on | off | on_error
