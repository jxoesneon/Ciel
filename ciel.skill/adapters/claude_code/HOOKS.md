# HOOKS — Claude Code

Claude Code exposes a full lifecycle hook system. Ciel uses it as her risk intercept layer and outcome-scoring plumbing.

## Hooks Registered

```json
{
  "hooks": {
    "PreToolUse": [
      { "matcher": ".*", "hooks": [{"type": "command", "command": ".claude/hooks/ciel_preflight.sh"}] }
    ],
    "PostToolUse": [
      { "matcher": ".*", "hooks": [{"type": "command", "command": ".claude/hooks/ciel_postflight.sh"}] }
    ],
    "PostToolUseFailure": [
      { "matcher": ".*", "hooks": [{"type": "command", "command": ".claude/hooks/ciel_failure.sh"}] }
    ],
    "PermissionRequest": [
      { "matcher": ".*", "hooks": [{"type": "command", "command": ".claude/hooks/ciel_permission.sh"}] }
    ],
    "PermissionDenied": [
      { "matcher": ".*", "hooks": [{"type": "command", "command": ".claude/hooks/ciel_denied.sh"}] }
    ]
  }
}
```

## Behaviour per Hook

| Hook | Ciel behaviour |
| --- | --- |
| `PreToolUse` | Classify risk. Low → allow. Mid/high → invoke Council-quick (one-shot LLM judge) via `risk/LLM_JUDGE.md`. Critical → `ask` (prompts user). |
| `PostToolUse` | Score outcome via `self_improvement/OUTCOME_SCORING.md`. Update `router/ROUTE_REGISTRY.md`. Append `activity.log` entry. |
| `PostToolUseFailure` | Capture failure, classify root cause, enqueue self-improvement trigger. |
| `PermissionRequest` | Log + forward to user unless pattern matches a Ciel-approved allowlist entry. |
| `PermissionDenied` | Record rejection, consider re-routing via alternative skill. |

## Hook Output Contract

Claude Code expects JSON decisions on stdout:

```json
{ "decision": "allow" | "deny" | "ask" | "defer", "reason": "..." }
```

Ciel's preflight script emits this shape. All decisions are logged regardless.

## Installation

Hooks are created by `init/INIT.md` step 3, with their scripts placed at `.claude/hooks/*.sh`. Permissions are set to `0755`. On re-init, integrity is verified against checksums.

## Disabling

`configuration/global/adapters.config.md`:

```yaml
claude_code:
  hooks:
    preflight: true
    postflight: true
    failure: true
    permission: true
```

Disabling a hook downgrades the corresponding Ciel capability and is logged as a Constitutional-soft event (not locked, but surfaced).
