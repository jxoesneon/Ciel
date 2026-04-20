# DEBUG

Debug mode. Verbose logging + trace format + activation flags.

## Activation

- Env var `CIEL_DEBUG=1`.
- Slash command `/ciel --debug`.
- Config `observability.config.log_verbosity: trace`.

Debug mode persists for the session.

## What Changes

- `activity.log` adds `trace` entries with full inputs/outputs (still secret-redacted).
- OTEL sampling goes to 100% regardless of configured rate.
- Router logs every candidate considered with scores.
- Council logs Stage 1 raw rationales (even for passing votes).
- Cache hit/miss stats per call.
- Memory op traces at key level.

## Trace Format

Extended `activity.log` entry:

```json
{
  "kind": "trace",
  "phase": "router.fast_path.scoring",
  "inputs": { "request_hash": "...", "project": "..." },
  "candidates": [ { "id": "...", "score": 0.87, "tags_matched": 3 } ],
  "elapsed_us": 4830,
  "memory_ops": 2
}
```

## Secrets

Still redacted in debug mode. Debug never unlocks secret exfiltration.

## Performance

Debug adds ~5–15% overhead per op. Not recommended for steady-state.

## Visualization

Seed skill `log_analyzer/SKILL.md` provides `--trace` mode to render debug traces as call graphs (ASCII or export to Chrome Trace Viewer JSON).
