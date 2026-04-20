# COMPUTER_USE — Claude Code

Claude Code can drive a desktop (research preview as of authoring). Ciel routes UI-automation requests here when the capability flag is `preview` or `true`.

## Activation

Set via `configuration/global/adapters.config.md`:

```yaml
claude_code:
  computer_use: preview   # or: true | false
```

Ciel refuses computer-use unless the flag is non-false **and** the current operation passed the risk classifier at ≤ mid.

## Typical Routes

- Visual verification of deployed UI (QA + screenshot diff).
- Native-app interaction where no CLI equivalent exists.
- Copying from apps with no API (read only).

## Constraints

- All clicks/keystrokes are bounded by a declared target region; free-ranging clicking is banned by default (Safety member veto).
- Every computer-use session is preceded by a screenshot captured and stored in the local `.ciel/traces/` directory for audit.
- User is notified before the first computer-use action per session and may decline.

## Integration

Delegated to `ultramac-mcp` when available (see MCP configuration). Ciel prefers MCP-exposed computer control over Claude Code's built-in preview when both are available, because MCP interactions are uniformly logged.

## Failure

If a planned UI action cannot complete (element missing, dialog, auth prompt), Ciel halts, captures state, and escalates. Never persists through a confusing state — the Safety member's core stance.
