# MODEL_ROUTING — Gemini CLI

Gemini CLI supports automatic model fallback: if a preferred model is unavailable or throttled, it drops to the next. Ciel respects the router but overlays her own policy.

## Ciel's Model Preferences by Path

| Path | Preferred | Fallback |
| --- | --- | --- |
| Fast path (deterministic) | n/a (no model call) | n/a |
| Reasoning path | gemini-3-pro | gemini-3-flash |
| Acquisition composition | gemini-3-pro | gemini-3-flash |
| Council member (Safety, Evolution) | gemini-3-pro | gemini-3-flash |
| Council member (Coherence, Efficiency) | gemini-3-flash | gemini-2.5-flash |
| Council member (Capability) | gemini-3-pro | gemini-3-flash |
| Risk classifier | gemini-3-flash | gemini-2.5-flash |
| Outcome scoring | gemini-3-flash | gemini-2.5-flash |

## Degradation Handling

When the primary is unavailable and Ciel falls back:

- log the fallback event,
- increase the confidence floor for that path by `model_routing.fallback_confidence_bump` (default 0.05) since weaker models should clear a higher bar,
- tag any resulting self-improvement proposal with `degraded_model: true`.

## Budget

```yaml
model_routing:
  budget_per_session_usd: 3.00
  alert_on: budget_fraction >= 0.8
  hard_stop_on: budget_fraction >= 1.0
```

On hard stop, Ciel escalates immediately and runs only fast path until the user adjusts or session ends.
