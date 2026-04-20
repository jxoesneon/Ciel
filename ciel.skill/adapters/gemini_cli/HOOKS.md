# HOOKS — Gemini CLI

Gemini CLI exposes a hook system where lifecycle events can be customized via scripts.

## Events Ciel Hooks

| Event | Script | Purpose |
| --- | --- | --- |
| `tool.preinvoke` | `.gemini/hooks/ciel_preflight.sh` | risk classification + allow/deny/ask |
| `tool.postinvoke` | `.gemini/hooks/ciel_postflight.sh` | outcome scoring |
| `tool.error` | `.gemini/hooks/ciel_failure.sh` | failure capture + self-improvement trigger |
| `permission.request` | `.gemini/hooks/ciel_permission.sh` | audit + allow-list check |
| `session.start` | `.gemini/hooks/ciel_session_start.sh` | verify integrity + load local context |
| `session.end` | `.gemini/hooks/ciel_session_end.sh` | flush activity log, compact traces |

## Output Contract

Scripts emit JSON on stdout:

```json
{ "decision": "allow" | "deny" | "ask", "reason": "...", "annotations": { ... } }
```

## Plan Mode Integration

Preflight hook checks if Plan mode is currently active. In Plan mode, `decision` is forced to `deny` for any write / destructive tool, surfacing a message so the user sees what *would* have happened — a natural pre-flight.

## Installation

Written by `init/INIT.md` step 3. Scripts set to `0755`, checksums in `~/.ciel/INTEGRITY.json`.

## Disabling

Per `configuration/global/adapters.config.md`:

```yaml
gemini_cli:
  hooks:
    preinvoke: true
    postinvoke: true
    error: true
    permission: true
    session: true
```
