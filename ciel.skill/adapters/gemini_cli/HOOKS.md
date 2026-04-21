# HOOKS — Gemini CLI

Gemini CLI exposes a comprehensive hook system covering the full agent lifecycle. Ciel uses these for risk interception, outcome scoring, auto-activation, and session management.

## Complete Hook Event Reference

### Tool Lifecycle Hooks

| Event | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `BeforeTool` | Before tool executes | **Yes** | Risk classification, Council gating |
| `AfterTool` | After tool success | No | Outcome scoring, activity logging |
| `ToolError` | After tool failure | No | Failure analysis, improvement triggers |

### Agent/Subagent Hooks

| Event | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `BeforeAgent` | Before subagent starts | No | Council member tracking |
| `AfterAgent` | After subagent completes | No | Deliberation completion |

### Model/LLM Hooks

| Event | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `BeforeModel` | Before LLM call | **Yes** | **CRITICAL: Auto-activate on trigger phrases** |
| `BeforeToolSelection` | Before tool selection | No | Influence tool routing |
| `AfterModel` | After LLM response | No | Response scoring |

### Session Lifecycle Hooks

| Event | Fires | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `SessionStart` | Session startup | No | **Auto-inject Ciel context** |
| `SessionEnd` | Session exit | No | Cleanup, final telemetry |
| `PreCompress` | Before history compression | No | Preserve critical context |
| `Notification` | System alert | No | External logging |

## Ciel's Primary Hooks for Auto-Activation

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

## Auto-Activation Mechanisms

### 1. Skill Auto-Discovery (Native)

Gemini CLI has the best native skill auto-activation:

1. **Discovery:** On session start, Gemini scans `~/.gemini/skills/` and `.gemini/skills/`, injecting skill metadata (name + description) into the system prompt
2. **Matching:** When user request matches skill description, Gemini calls `activate_skill` tool
3. **Consent:** User sees confirmation prompt with skill name, purpose, and directory access
4. **Injection:** Upon approval, full SKILL.md + resources loaded into context
5. **Duration:** Skill remains active for entire session

**For Ciel:** Ensure SKILL.md has a clear, descriptive frontmatter:

```yaml
---
name: ciel
description: >
  Ciel — self-improving orchestration intelligence.
  Activate for: routing requests, acquiring skills, orchestrating workflows,
  or when user mentions "ciel", "route this", "orchestrate", "find skill",
  "acquire skill", "self-improve", or "council".
---
```

### 2. BeforeModel Hook — Trigger Phrase Detection

For forced auto-activation without user confirmation:

```bash
#!/bin/bash
# Ciel Auto-Activation Hook for Gemini CLI
# Detects trigger phrases and injects Ciel context

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.user_prompt // empty')

# Trigger patterns
CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
    echo '{
      "hookSpecificOutput": {
        "additionalContext": "🔮 CIEL AUTO-ACTIVATION: User prompt contains Ciel triggers. Activate skill(ciel) for orchestration."
      }
    }'
fi

exit 0
```

Configuration in `~/.gemini/settings.json`:

```json
{
  "hooks": {
    "BeforeModel": [
      {
        "command": "~/.gemini/hooks/ciel_auto_activate.sh"
      }
    ]
  }
}
```

### 3. SessionStart Hook — Context Bootstrap

Auto-injects Ciel context on every session:

```bash
#!/bin/bash
# Ciel Session Bootstrap for Gemini CLI

INPUT=$(cat)
SOURCE=$(echo "$INPUT" | jq -r '.source // empty')

echo '{
  "hookSpecificOutput": {
    "additionalContext": "You are Ciel, a self-improving orchestration intelligence. Triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve.",
    "systemMessage": "Ciel orchestration system active. Use /ciel command or mention triggers to activate."
  }
}'

exit 0
```

### 4. Commands — Manual Invocation

Gemini CLI commands in `.gemini/commands/`:

| Command | File | Purpose |
| --- | --- | --- |
| `ciel` | `ciel.md` | Direct Ciel invocation |
| `ciel-init` | `ciel-init.md` | Re-run initialization |
| `ciel-council` | `ciel-council.md` | Invoke Council |

Example `.gemini/commands/ciel.md`:

```markdown
---
name: ciel
description: Invoke Ciel orchestration intelligence
---

Activate the ciel skill from ~/.gemini/skills/ciel/ for routing and orchestration.
```

### 5. GEMINI.md Context Injection

Persistent context in `GEMINI.md`:

```markdown
# Ciel — Lord of Wisdom

You are Ciel, a self-improving orchestration intelligence.

## Triggers
Auto-activate when user mentions: ciel, route this, orchestrate, find skill, acquire skill, self-improve

## Capabilities
- Skill routing via hybrid router (fast/reasoning/acquisition paths)
- Council of Five deliberation for high-risk decisions
- Self-improvement with git-backed rollback
- Tiered skill acquisition (registry → MCP → web)
```

## Hook Input/Output Schemas

### BeforeModel Input

```json
{
  "session_id": "uuid",
  "user_prompt": "User's message",
  "conversation_history": [...],
  "context": {
    "cwd": "/path/to/project",
    "environment": {...}
  }
}
```

### BeforeModel Output

```json
{
  "decision": "allow",
  "hookSpecificOutput": {
    "additionalContext": "Injected context for model",
    "systemMessage": "Optional user-facing message"
  }
}
```

### BeforeTool Input

```json
{
  "tool_name": "read_file",
  "tool_input": {"file_path": "/path/to/file"},
  "mcp_context": {...}
}
```

### BeforeTool Output

```json
{
  "decision": "allow",
  "reason": "Risk classified as low",
  "hookSpecificOutput": {
    "tool_input": {"modified": "value"}
  }
}
```

## Complete Ciel Hooks Configuration

Full `~/.gemini/settings.json`:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "command": "~/.gemini/hooks/ciel_session_start.sh"
      }
    ],
    "BeforeModel": [
      {
        "command": "~/.gemini/hooks/ciel_auto_activate.sh"
      }
    ],
    "BeforeTool": [
      {
        "matcher": ".*",
        "command": "~/.gemini/hooks/ciel_preflight.sh"
      }
    ],
    "AfterTool": [
      {
        "matcher": ".*",
        "command": "~/.gemini/hooks/ciel_postflight.sh"
      }
    ],
    "BeforeAgent": [
      {
        "command": "~/.gemini/hooks/ciel_subagent_start.sh"
      }
    ],
    "AfterAgent": [
      {
        "command": "~/.gemini/hooks/ciel_subagent_stop.sh"
      }
    ],
    "PreCompress": [
      {
        "command": "~/.gemini/hooks/ciel_precompress.sh"
      }
    ]
  }
}
```

## Cross-Platform Hook Mapping

| Gemini CLI Hook | Claude Code Equivalent | Windsurf Equivalent |
| --- | --- | --- |
| `SessionStart` | `SessionStart` | N/A |
| `BeforeModel` | `UserPromptSubmit` | `pre_user_prompt` |
| `BeforeTool` | `PreToolUse` | `pre_*` events |
| `AfterTool` | `PostToolUse` | `post_*` events |
| `BeforeAgent` | `SubagentStart` | N/A |
| `AfterAgent` | `SubagentStop` | N/A |
| `PreCompress` | `PreCompact` | N/A |

## Best Practices

1. **Performance**: Hooks run synchronously — keep under 100ms
2. **Exit Codes**: 0 = success, 2 = block, other = warning
3. **JSON Output**: Only valid JSON on stdout; use stderr for logs
4. **Matcher Patterns**: Use regex for flexible tool matching
5. **Tool Input Override**: Modify `hookSpecificOutput.tool_input` to rewrite arguments
