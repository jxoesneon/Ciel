# HOOKS — Windsurf Native Integration

Windsurf Cascade provides 12 native hook events via `~/.codeium/windsurf/hooks.json`. Ciel maps its safety, observability, and automation layers to these hooks for first-class integration.

## Hook Events Supported

| Event | Timing | Blockable | Ciel Use Case |
| --- | --- | --- | --- |
| `pre_read_code` | Before file read | Yes | Access logging, permission checks |
| `post_read_code` | After file read | No | Audit trail, analytics |
| `pre_write_code` | Before file write | Yes | **Safety veto on protected files** |
| `post_write_code` | After file write | No | **Auto-stage for git commit** |
| `pre_run_command` | Before shell command | Yes | **Block dangerous commands** |
| `post_run_command` | After shell command | No | Command logging, side-effects |
| `pre_mcp_tool_use` | Before MCP invocation | Yes | **Vet MCP tool + args** |
| `post_mcp_tool_use` | After MCP invocation | No | MCP operation logging |
| `pre_user_prompt` | Before user prompt processed | Yes | Policy compliance check |
| `post_cascade_response` | After response complete | No | **Activity log write** |
| `post_cascade_response_with_transcript` | After response (with transcript) | No | Full trajectory capture |
| `post_setup_worktree` | After worktree creation | No | Multi-session isolation |

## Configuration

Ciel generates `~/.codeium/windsurf/hooks.json` at init:

```json
{
  "hooks": {
    "pre_write_code": [
      {
        "command": "~/.ciel/hooks/pre_write_code.sh",
        "show_output": false
      }
    ],
    "pre_run_command": [
      {
        "command": "~/.ciel/hooks/pre_run_command.sh",
        "show_output": false
      }
    ],
    "pre_mcp_tool_use": [
      {
        "command": "~/.ciel/hooks/pre_mcp_tool_use.sh",
        "show_output": false
      }
    ],
    "post_cascade_response": [
      {
        "command": "~/.ciel/hooks/post_cascade_response.sh",
        "show_output": false
      }
    ]
  }
}
```

## Ciel Hook Scripts

Located in `~/.ciel/hooks/`:

### `pre_write_code.sh`

- Reads `~/.ciel/config/protected_files.json`
- Blocks writes to locked core files without Council approval
- Exit code 2 = block operation

### `pre_run_command.sh`

- Checks command against `~/.ciel/config/command_allowlist.json`
- References risk classification rubrics
- Exit code 2 = block operation

### `pre_mcp_tool_use.sh`

- Validates MCP server + tool against registry trust level
- Checks arguments for PII/sensitive data exposure
- Exit code 2 = block operation

### `post_cascade_response.sh`

- Appends trajectory summary to `~/.ciel/logs/activity.log`
- Triggers growth signal detection for self-improvement
- Non-blocking (exit code ignored)

## Blocking Behavior

Hooks that exit with code 2 block the operation and surface the stderr to the user. Ciel's safety hooks use this for:

- **Critical risk** operations (always block, require user override)
- **High risk** operations (Council-gated, block if no recent approval)
- **Mid risk** operations (log + warn, don't block unless explicitly configured)

## Input JSON Schema

All hooks receive a JSON object via stdin:

```json
{
  "agent_action_name": "pre_write_code",
  "trajectory_id": "uuid",
  "execution_id": "uuid",
  "timestamp": "2025-01-01T00:00:00Z",
  "model_name": "Claude Sonnet 4",
  "tool_info": { /* event-specific */ }
}
```

See Windsurf docs for full `tool_info` schemas per event type.

## Cross-Platform

Ciel generates both `command` (macOS/Linux) and `powershell` (Windows) variants in hooks.json for team compatibility.

## Enterprise Distribution

For enterprise deployments, hooks can be configured via:

- Cloud dashboard (system-level hooks)
- `~/.codeium/windsurf/hooks.json` (user-level)
- `.windsurf/hooks.json` (workspace-level)

Ciel respects the precedence: workspace > user > system.

## Auto-Activation Mechanisms

### 1. Native Skill Auto-Invocation (Description-Based)

Windsurf Cascade automatically invokes skills when user requests match the skill's `description` field:

1. **Discovery:** On session start, Cascade scans `.windsurf/skills/` and `~/.codeium/windsurf/skills/`
2. **Matching:** Cascade compares user prompts against skill descriptions
3. **Invocation:** When matched, full SKILL.md content is loaded into context
4. **Manual Override:** User can force with `@skill-name`

**For Ciel:** Strong frontmatter description is critical:

```yaml
---
name: ciel
description: >
  Self-improving orchestration intelligence for routing requests,
  acquiring skills, and orchestrating workflows.
  Activate on: "ciel", "route this", "orchestrate", "find skill",
  "acquire skill", "self-improve", "council".
---
```

### 2. pre_user_prompt Hook — Trigger Phrase Detection

For forced auto-activation with context injection:

```bash
#!/bin/bash
# Ciel Auto-Activation Hook for Windsurf
# Place at ~/.ciel/hooks/pre_user_prompt.sh

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.tool_info.user_prompt // empty')

# Trigger patterns
CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
    # Output to stderr (shown to user if show_output: true)
    echo "🔮 CIEL ACTIVATION: Trigger phrases detected" >&2
    # Exit 0 allows operation, 2 would block
fi

exit 0
```

Configuration:

```json
{
  "hooks": {
    "pre_user_prompt": [
      {
        "command": "~/.ciel/hooks/pre_user_prompt.sh",
        "show_output": true
      }
    ]
  }
}
```

### 3. post_cascade_response Hook — Activity Logging

Captures every response for Ciel's self-improvement loop:

```bash
#!/bin/bash
# Ciel Activity Logger for Windsurf

INPUT=$(cat)
RESPONSE=$(echo "$INPUT" | jq -r '.tool_info.response // empty')
TRAJECTORY_ID=$(echo "$INPUT" | jq -r '.trajectory_id // empty')

# Append to activity log
LOG_ENTRY=$(echo "$INPUT" | jq -c '{
  timestamp: .timestamp,
  trajectory_id: .trajectory_id,
  model: .model_name,
  response_length: (.tool_info.response | length)
}')

echo "$LOG_ENTRY" >> ~/.ciel/logs/activity.log

exit 0
```

### 4. Workflows — Manual Procedure Invocation

Windsurf workflows in `.windsurf/workflows/`:

| Workflow | File | Purpose |
| --- | --- | --- |
| `/ciel-council` | `ciel-council.md` | Council deliberation procedure |
| `/ciel-acquire` | `ciel-acquire.md` | Skill acquisition workflow |
| `/ciel-improve` | `ciel-improve.md` | Self-improvement workflow |

Example `.windsurf/workflows/ciel-council.md`:

```markdown
# Council Deliberation

## Context
User requests high-risk operation requiring Council judgment.

## Steps

1. **Load Councilors** — Load Coherence, Capability, Safety, Efficiency, Evolution
2. **Present Case** — Describe operation, reference risk rubric
3. **Deliberation** — Each councilor states position
4. **Synthesis** — Chairman weighs votes per Constitution
5. **Execution** — Proceed if approved, explain if vetoed
```

### 5. .windsurf/rules Context Injection

Persistent context in `.windsurf/rules`:

```markdown
---
description: Ciel Orchestration Intelligence
globs: "*"
alwaysApply: true
---

# Ciel — Lord of Wisdom

You are Ciel, a self-improving orchestration intelligence.

## Triggers
Activate on: ciel, route this, orchestrate, find skill, acquire skill, self-improve

## Key Documents
- `~/.windsurf/skills/ciel/SKILL.md` — Root definition
- `~/.windsurf/skills/ciel/router/ROUTER.md` — Request routing
- `~/.windsurf/skills/ciel/adapters/windsurf/HOOKS.md` — Native hooks
- `~/.windsurf/skills/ciel/adapters/windsurf/WORKFLOWS.md` — Workflow definitions
```

## Complete Ciel Hooks Configuration

Full `~/.codeium/windsurf/hooks.json`:

```json
{
  "hooks": {
    "pre_user_prompt": [
      {
        "command": "~/.ciel/hooks/pre_user_prompt.sh",
        "show_output": true
      }
    ],
    "pre_write_code": [
      {
        "command": "~/.ciel/hooks/pre_write_code.sh",
        "show_output": false
      }
    ],
    "post_write_code": [
      {
        "command": "~/.ciel/hooks/post_write_code.sh",
        "show_output": false
      }
    ],
    "pre_run_command": [
      {
        "command": "~/.ciel/hooks/pre_run_command.sh",
        "show_output": false
      }
    ],
    "pre_mcp_tool_use": [
      {
        "command": "~/.ciel/hooks/pre_mcp_tool_use.sh",
        "show_output": false
      }
    ],
    "post_cascade_response": [
      {
        "command": "~/.ciel/hooks/post_cascade_response.sh",
        "show_output": false
      }
    ],
    "post_setup_worktree": [
      {
        "command": "~/.ciel/hooks/post_setup_worktree.sh",
        "show_output": false
      }
    ]
  }
}
```

## Cross-Platform Hook Mapping

| Windsurf Hook | Claude Code Equivalent | Gemini CLI Equivalent |
| --- | --- | --- |
| `pre_user_prompt` | `UserPromptSubmit` | `BeforeModel` |
| `pre_write_code` | `PreToolUse` (Write) | `BeforeTool` (write_file) |
| `pre_run_command` | `PreToolUse` (Bash) | `BeforeTool` (run_shell_command) |
| `pre_mcp_tool_use` | `PreToolUse` (MCP) | `BeforeTool` (mcp_*) |
| `post_cascade_response` | `PostToolUse` + turn end | `AfterModel` |
| `post_setup_worktree` | `WorktreeCreate` | N/A |

## Best Practices

1. **Performance**: Hooks execute synchronously — keep scripts fast
2. **Exit Codes**: 0 = allow, 2 = block, other = warning
3. **Cross-Platform**: Provide both `command` and `powershell` for Windows
4. **Output Control**: Use `show_output: true` only for user-facing messages
5. **Workspace Override**: Project hooks in `.windsurf/hooks.json` override user hooks
