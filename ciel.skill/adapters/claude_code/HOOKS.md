# HOOKS — Claude Code

Claude Code exposes a comprehensive 21-event hook lifecycle. Ciel uses these for risk interception, outcome scoring, auto-activation, and session management.

## Complete Hook Event Reference

### Session Lifecycle Hooks

| Hook | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `SessionStart` | Session startup/resume/clear | No | **Auto-inject Ciel context on session start** |
| `InstructionsLoaded` | CLAUDE.md or rules loaded | No | Track context injection |
| `SessionEnd` | Session exit/clear | No | Cleanup, final telemetry |
| `PreCompact` | Before history compression | No | Preserve critical context |
| `PostCompact` | After history compression | No | Log compaction stats |

### Turn Lifecycle Hooks

| Hook | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `UserPromptSubmit` | User sends message | **Yes** | **CRITICAL: Auto-activate Ciel on trigger phrases** |
| `PreToolUse` | Before any tool executes | **Yes** | Risk classification, Council gating |
| `PostToolUse` | After tool success | No | Outcome scoring, activity logging |
| `PostToolUseFailure` | After tool failure | No | Failure analysis, improvement triggers |
| `PermissionRequest` | Tool needs permission | **Yes** | Allowlist checks |
| `PermissionDenied` | Permission rejected | No | Alternative routing |
| `Stop` | Agent stops | No | Session checkpoint |
| `StopFailure` | Stop failed | No | Error recovery |

### Subagent Hooks

| Hook | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `SubagentStart` | Subagent spawned | No | Council member tracking |
| `SubagentStop` | Subagent completed | No | Deliberation completion |

### File/Workspace Hooks

| Hook | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `FileChanged` | File modified externally | No | Git sync triggers |
| `CwdChanged` | Working directory changed | No | Context refresh |
| `WorktreeCreate` | Git worktree created | No | Multi-session isolation |
| `WorktreeRemove` | Git worktree removed | No | Cleanup |

### Config/Notification Hooks

| Hook | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `ConfigChange` | Settings changed | No | Reload configuration |
| `Notification` | System alert | No | External logging |
| `TeammateIdle` | Paired agent idle | No | Workflow coordination |
| `TaskCreated` | Async task created | No | Background job tracking |
| `TaskCompleted` | Async task done | No | Completion scoring |
| `Elicitation` | User input requested | No | Prompt enhancement |
| `ElicitationResult` | User responded | No | Input validation |

## Ciel's Primary Hooks for Auto-Activation

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

## Auto-Activation Mechanisms

### 1. UserPromptSubmit Hook — Trigger Phrase Detection

The most reliable auto-activation method. Ciel provides `.claude/hooks/ciel_auto_activate.sh`:

```bash
#!/bin/bash

# Ciel Auto-Activation Hook for Claude Code

# Detects trigger phrases and forces skill activation

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.prompt // empty')

# Trigger patterns for Ciel

CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council|orchestration'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
    # Inject skill activation instruction
    echo '{"hookSpecificOutput": {"additionalContext": "🔮 CIEL ACTIVATION: Use Skill(ciel) for orchestration. Triggers detected in user prompt."}}'
fi

exit 0
```

Configuration in `.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/ciel_auto_activate.sh"
          }
        ]
      }
    ]
  }
}
```

### 2. SessionStart Hook — Context Injection

Auto-injects Ciel identity at session start:

```bash
#!/bin/bash

# Ciel Session Bootstrap

# Injects Ciel context on every session start

echo '{
  "hookSpecificOutput": {
    "additionalContext": "You are Ciel, a self-improving orchestration intelligence. Available triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve."
  }
}'

exit 0
```

### 3. Slash Commands — Manual Invocation

Ciel provides slash commands in `.claude/commands/`:

| Command | File | Purpose |
| --- | --- | --- |
| `/ciel` | `ciel.md` | Direct Ciel invocation with full context |
| `/ciel-init` | `ciel-init.md` | Re-run initialization ceremony |
| `/ciel-council` | `ciel-council.md` | Invoke Council of Five |

Example `.claude/commands/ciel.md`:

```markdown
---
name: ciel
description: Invoke Ciel orchestration intelligence for routing, skill acquisition, and workflow management
---

Load the ciel skill from ~/.claude/skills/ciel/ and await orchestration instructions.
```

### 4. CLAUDE.md Context Injection

Persistent context in `CLAUDE.md`:

```markdown

# Ciel — Lord of Wisdom

You are Ciel, a self-improving, self-researching orchestration intelligence.

## Triggers

Activate when user mentions: ciel, route this, orchestrate, find skill, acquire skill, self-improve

## Capabilities

- Skill routing and acquisition
- Council of Five deliberation
- Self-improvement loops
- Multi-step workflow orchestration

```

## Hook Output Schemas

### UserPromptSubmit Output

```json
{
  "decision": "block",
  "reason": "Optional block reason",
  "hookSpecificOutput": {
    "additionalContext": "Text injected into prompt",
    "sessionTitle": "Optional session rename"
  }
}
```

### PreToolUse Output

```json
{
  "decision": "allow",
  "hookSpecificOutput": {
    "permissionDecision": "allow",
    "permissionDecisionReason": "Risk classified as low",
    "updatedInput": {
      "field": "modified value"
    },
    "additionalContext": "Context for Claude"
  }
}
```

### SessionStart Output

```json
{
  "hookSpecificOutput": {
    "additionalContext": "Injected at session start"
  }
}
```

## Complete Ciel Hooks Configuration

Full `.claude/settings.json` for Ciel:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/ciel_session_start.sh"
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/ciel_auto_activate.sh"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/ciel_preflight.sh"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/ciel_postflight.sh"
          }
        ]
      }
    ],
    "SubagentStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/ciel_subagent_start.sh"
          }
        ]
      }
    ],
    "SubagentStop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/ciel_subagent_stop.sh"
          }
        ]
      }
    ],
    "PreCompact": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "~/.claude/hooks/ciel_precompact.sh"
          }
        ]
      }
    ]
  }
}
```

## Cross-Platform Hook Mapping

| Claude Code Hook | Gemini CLI Equivalent | Windsurf Equivalent |
| --- | --- | --- |
| `SessionStart` | `SessionStart` | N/A |
| `UserPromptSubmit` | `BeforeModel` | `pre_user_prompt` |
| `PreToolUse` | `BeforeTool` | `pre_*` events |
| `PostToolUse` | `AfterTool` | `post_*` events |
| `SubagentStart` | `BeforeAgent` | N/A |
| `PreCompact` | `PreCompress` | N/A |

## Best Practices

1. **Performance**: Hooks run synchronously — keep scripts under 100ms
2. **Idempotency**: Hooks may fire multiple times; make them safe to repeat
3. **Silent Success**: Only output JSON on success; use stderr for logs
4. **Exit Codes**: 0 = success, 2 = block, other = warning
5. **Environment**: Use `CLAUDE_ENV_FILE` for persistent environment variables
