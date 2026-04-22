# CHECKPOINTS — Windsurf Named Checkpoint Integration

Windsurf Cascade provides **Named Checkpoints and Reverts** — a native state management system that allows creating named snapshots of project state and reverting to them at any time.

## Overview

Named Checkpoints provide:

- **Point-in-time snapshots** of the entire codebase state
- **Named references** for important milestones (e.g., "before-refactor", "v1.0-stable")
- **Irreversible reverts** to previous states
- **UI-native integration** via Cascade conversation panel

## How Checkpoints Work

### Creating a Checkpoint

Users can create named checkpoints from within the Cascade conversation:

1. Hover over any prompt in the Cascade panel
2. Click the checkpoint/snapshot icon
3. Provide a descriptive name
4. Checkpoint is saved and appears in the conversation timeline

### Reverting to a Checkpoint

1. Hover over the original prompt in Cascade panel
2. Click the revert arrow on the right
3. Or revert directly from the table of contents
4. **Warning**: Reverts are **irreversible** — all changes after that point are lost

## Ciel Integration Strategy

### 1. Checkpoint-Aware High-Risk Operations

Ciel can recommend creating a checkpoint before high-risk operations:

**Pre-flight Checklist (via `pre_run_command.sh` hook):**

- Detect high-risk commands (`rm -rf`, `git reset --hard`, mass refactoring)
- Prompt user to create a named checkpoint
- Block operation until checkpoint confirmed or user overrides

```bash
#!/bin/bash
# Ciel Checkpoint Recommendation Hook

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_info.command // empty')

# High-risk patterns
RISK_PATTERNS='rm -rf|git reset --hard|git clean -fd|git checkout --\\.|DROP TABLE|DELETE FROM'

if echo "$COMMAND" | grep -qiE "$RISK_PATTERNS"; then
    echo "⚠️  HIGH-RISK OPERATION DETECTED" >&2
    echo "Ciel recommends creating a named checkpoint before proceeding." >&2
    echo "Revert is irreversible — checkpoint ensures you can return to this state." >&2
fi

exit 0
```

### 2. Checkpoint-Based Council Deliberation

For Council-gated operations, checkpoints serve as **pre-approval snapshots**:

**Workflow:**

1. User requests high-risk operation
2. Ciel invokes Council of Five
3. **Before deliberation**: Ciel suggests creating checkpoint "pre-council-[timestamp]"
4. Council deliberates
5. If approved: proceed with operation
6. If vetoed: user can revert to checkpoint

### 3. Self-Improvement Recovery Points

Ciel can use checkpoints as **recovery points** during self-improvement:

**Pattern:**

```markdown
## Self-Improvement with Checkpoint Recovery

1. **Pre-mutation checkpoint** — Create "pre-improvement-[id]"
2. **Apply improvement** — Execute the proposed change
3. **Observe** — Monitor subsequent interactions
4. **Rollback if needed** — Revert to checkpoint on regression
```

### 4. Named Checkpoint Conventions

Ciel establishes naming conventions for consistent checkpoint management:

| Prefix | Purpose | Example |
| --- | --- | --- |
| `pre-council-` | Before Council deliberation | `pre-council-2025-01-15` |
| `pre-improvement-` | Before self-improvement | `pre-improvement-risk-model` |
| `pre-acquire-` | Before skill acquisition | `pre-acquire-docker-skill` |
| `stable-` | Known good state | `stable-v1.2.0` |
| `experiment-` | Experimental branch | `experiment-async-router` |

### 5. Checkpoint Integration with Activity Log

Ciel logs checkpoint events for audit trail:

```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "event": "checkpoint_created",
  "checkpoint_name": "pre-council-risk-model-update",
  "trigger": "ciel_safety_recommendation",
  "context": "high_risk_operation_detected"
}
```

## Cross-Platform Comparison

| Feature | Claude Code | Gemini CLI | Windsurf Checkpoints |
| --- | --- | --- | --- |
| **State snapshots** | Git worktrees | Git branches | Native checkpoints |
| **Named references** | Worktree names | Branch names | Checkpoint names |
| **Revert capability** | `git checkout` | `git checkout` | Native revert UI |
| **Reversible** | Yes | Yes | **No (irreversible)** |
| **Scope** | Git worktree | Repository | Conversation + codebase |

## Best Practices

### For Users

1. **Always checkpoint before high-risk operations**
2. **Use descriptive names** — Include date, purpose, context
3. **Checkpoint hierarchy** — Major milestones get "stable-" prefix
4. **Clean up old checkpoints** — Conversation checkpoints accumulate

### For Ciel

1. **Recommend checkpoints proactively** — Before any destructive operation
2. **Document checkpoint names** — In activity logs for audit trail
3. **Link to Council decisions** — Checkpoint names reference Council votes
4. **Respect irreversibility** — Emphasize that revert cannot be undone

## Implementation Status

**Current**: Documentation and hook scripts defined
**Future Enhancement**: Direct checkpoint API integration (if Windsurf exposes programmatic checkpoint creation)

## Related Documents

- `adapters/windsurf/HOOKS.md` — Pre-tool safety hooks
- `council/COUNCIL.md` — Council deliberation workflows
- `self_improvement/REGRESSION_DETECTION.md` — Rollback procedures
