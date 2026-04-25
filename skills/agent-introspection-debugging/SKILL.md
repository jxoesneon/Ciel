---
name: agent-introspection-debugging
version: 1.0.0
format: skill/1.0
description: A structured self-debugging protocol for resolving agentic loops, failures, and reasoning drift.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(agent|self).*(debugging|introspection|looping|stuck)"
    confidence: 0.95
  - pattern: "why is the agent (failing|repeating|drifting)"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Agent Introspection Debugging (Self-Diagnosis)

This skill formalizes the "Self-Debug" protocol for CIEL sub-agents. It provides a systematic 4-phase loop to diagnose and recover from repetitive failures, tool-call loops, and reasoning drift before escalating to a human.

## The 4-Phase Loop

### 1. Failure Capture
Before retrying, the agent MUST record the failure state in a structured format:
- **Session Goal**: What was the intended outcome?
- **Last Meaningful Tool**: Which tool call preceded the failure?
- **Error Signature**: Exact error message or pattern seen.
- **Environment State**: cwd, branch, and any relevant system assumptions.

### 2. Root-Cause Diagnosis
Match the capture against common failure patterns:
- **Looping**: Repeated same tool call with same inputs.
- **Drift**: Reasoning has pivoted away from the original goal.
- **Stale State**: Working on files or assumptions that no longer match the filesystem.
- **Quota/Resource**: 429s, memory exhaustion, or context overflow.

### 3. Contained Recovery
Take the smallest reversible action to change the diagnosis surface:
- **Restate**: Re-initialize the task goal in a fresh context.
- **Verify**: Re-read the filesystem or run a diagnostic command (e.g., `ls`, `git status`).
- **Narrow**: Shrink the scope to a single failing test or file.
- **Escalate**: If the failure is high-risk (e.g., persistent auth failure), stop and ask the human.

### 4. Introspection Report
Log the debugging session to the MemPalace Diary (`mempalace_diary_write`):
- **Root Cause**: What was actually wrong?
- **Action Taken**: How was it resolved?
- **Prevention**: What change should be encoded (via `save_memory` or `coding-standards`) to prevent recurrence?

## Recovery Heuristics
1. **Evidence over Memory**: Never trust the prior thought trace if it leads to failure; verify the world state directly.
2. **Smallest Pivot**: Change one variable at a time when attempting recovery.
3. **Hard Stop**: After 3 repeated failures on the same sub-task, the agent MUST enter Phase 1.

## Anti-Patterns
- **Blind Retrying**: Repeating the same failing command with slightly different wording.
- **Guessing**: Proposing "fixes" to code without verifying the exact failure signature in logs.
- **Silent Failure**: Recovering from a significant error without logging an Introspection Report.
