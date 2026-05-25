# HOOKS — Devin for Terminal

Devin for Terminal uses a hook format **compatible with Claude Code hooks**. Ciel leverages this for risk interception, auto-activation, outcome scoring, and session management.

## Hook Event Reference (Devin)

| Event | When it fires | Blockable | Ciel Use |
|-------|--------------|-----------|----------|
| `PreToolUse` | Before any tool executes | **Yes** | Risk classification, Council gating |
| `PostToolUse` | After tool success | No | Outcome scoring, activity logging |
| `PermissionRequest` | Permission decision needed | **Yes** | Auto-approve safe patterns |
| `UserPromptSubmit` | User sends message | No | Trigger phrase detection, context injection |
| `Stop` | Agent wants to stop | **Yes** | Verify tests run before stopping |
| `SessionStart` | Session begins | No | Bootstrap Ciel identity |
| `SessionEnd` | Session ends | No | Cleanup, final telemetry |
| `PostCompaction` | After context compaction | No | Re-inject critical context |

## Where Hooks Live (Devin)

| Location | Scope | Format |
|----------|-------|--------|
| `.devin/hooks.v1.json` | Project | Standalone hooks file (entire file is the hooks object) |
| `.devin/config.json` | Project | `"hooks"` key inside config |
| `.devin/config.local.json` | Project local | `"hooks"` key (gitignored) |
| `~/.config/devin/config.json` | User global | `"hooks"` key |
| `.claude/settings.json` | Legacy | `"hooks"` key (loaded if `read_config_from.claude: true`) |

## Ciel's Recommended Devin Hook Config

### Project-Level: `.devin/hooks.v1.json`

```json
{
  "PreToolUse": [
    {
      "matcher": "",
      "hooks": [
        {
          "type": "command",
          "command": "~/.ciel/hooks/ciel_preflight.sh",
          "timeout": 5
        }
      ]
    }
  ],
  "PostToolUse": [
    {
      "matcher": "",
      "hooks": [
        {
          "type": "command",
          "command": "~/.ciel/hooks/ciel_postflight.sh",
          "timeout": 5
        }
      ]
    }
  ],
  "UserPromptSubmit": [
    {
      "matcher": "",
      "hooks": [
        {
          "type": "command",
          "command": "~/.ciel/hooks/ciel_auto_activate.sh",
          "timeout": 3
        }
      ]
    }
  ],
  "PermissionRequest": [
    {
      "matcher": "",
      "hooks": [
        {
          "type": "command",
          "command": "~/.ciel/hooks/ciel_permission.sh",
          "timeout": 5
        }
      ]
    }
  ]
}
```

### Global: `~/.config/devin/config.json`

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "~/.ciel/hooks/ciel_session_bootstrap.sh",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

## Hook Scripts

All scripts live in `~/.ciel/hooks/` and are shared across Claude Code, Gemini CLI, and Devin for Terminal:

| Script | Purpose |
|--------|---------|
| `ciel_preflight.sh` | Risk classification: block destructive, ask for sensitive writes |
| `ciel_postflight.sh` | Outcome scoring: log success/failure to `~/.ciel/activity.log` |
| `ciel_auto_activate.sh` | Detect trigger phrases and inject Ciel activation context |
| `ciel_session_bootstrap.sh` | Inject Ciel identity on every new session |
| `ciel_permission.sh` | Auto-approve read-only and known-safe build commands |

## Devin-Specific Hook Behaviours

### PermissionRequest + Devin Scope-Based Permissions

Devin has two permission systems that interact:

1. **Scope-based permissions** in `.devin/config.json` (e.g., `"allow": ["Read(src/**)"]`)
2. **Hook-based decisions** from PermissionRequest hooks

Order of precedence:
1. Devin checks `deny` rules first
2. Devin checks `ask` rules second
3. Devin checks `allow` rules third
4. If no rule matches, the PermissionRequest hook fires
5. If the hook returns no decision, Devin prompts the user

Ciel's `ciel_permission.sh` hook auto-approves at step 4 for known-safe commands, allowing Devin's scope-based rules to handle the bulk of permission management while Ciel adds intelligence for edge cases.

### PreToolUse + Devin Modes

Devin's permission modes affect when PreToolUse hooks fire:

| Mode | PreToolUse fires for... |
|------|------------------------|
| Normal | All tools (writes and execs prompt user first) |
| Accept Edits | All tools (edits auto-approved, execs still prompt) |
| Bypass | Hooks still fire, but decisions may be overridden |
| Autonomous (sandbox) | Hooks fire; sandbox enforces filesystem/network limits |
| Plan | Read-only only; hooks fire for reads |

Ciel's preflight hook works in all modes. In Bypass mode, the hook can still log and emit warnings even if it cannot block.

### Stop Hook — Devin Loop Awareness

Devin supports `/loop <prompt>` for automated review loops. Ciel's Stop hook should be aware of loop mode and not block indefinitely:

```json
{
  "Stop": [
    {
      "matcher": "",
      "hooks": [
        {
          "type": "command",
          "command": "~/.ciel/hooks/ciel_stop_check.sh",
          "timeout": 3
        }
      ]
    }
  ]
}
```

The stop check script verifies that tests have been run (if test tools are configured in the project) before allowing the agent to stop.

### PostCompaction — Context Preservation

Devin auto-compacts context when it grows large. Ciel's PostCompaction hook can re-inject critical identity and registry context that may have been lost:

```json
{
  "PostCompaction": [
    {
      "matcher": "",
      "hooks": [
        {
          "type": "command",
          "command": "~/.ciel/hooks/ciel_postcompact.sh",
          "timeout": 3
        }
      ]
    }
  ]
}
```

## Verification

Use Devin's `/hooks` slash command to see all loaded hooks, their source files, and IDs.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success — hook continues |
| 2 | Block — action is denied |
| Other | Error — logged but doesn't block |
