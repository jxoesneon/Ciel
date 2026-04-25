---
name: unified-notifications-ops
version: 1.0.0
format: skill/1.0
description: A hub for consolidating, classifying, and routing system and human notifications across multiple surfaces.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(unify|consolidate|route).*(notification|alert|ping)"
    confidence: 0.9
  - pattern: "what (is happening|happened) while I was (away|gone)"
    confidence: 0.85
---

# CIEL ADAPTATION: Unified Notifications (The Notification Hub)

This skill turns fragmented alerts into a single, actionable operator surface. It prevents "alert fatigue" by applying a strict severity model and routing logic.

## Severity Model
- **CRITICAL**: Broken CI on `main`, security alerts, failed deploys. *Action: Interrupt the Orchestrator immediately.*
- **HIGH**: Review requested, failing PR, owner-blocking tasks. *Action: Surface in the next turn summary.*
- **MEDIUM**: Issue state changes, comments, backlog movement. *Action: Group into a session-end digest.*
- **LOW**: Routine lifecycle markers, success pings. *Action: Log silently to MemPalace.*

## The Pipeline
1. **Capture**: Aggregate events from GitHub, Linear, local hooks, and system monitors.
2. **Classify**: Assign severity and owner based on the project context.
3. **Collapse**: Deduplicate identical events (e.g., the same PR update seen in GitHub and Email).
4. **Route**: Send to the most appropriate channel (Desktop alert, Digest, or Diary).

## Integration
- **Context Awareness**: Use `workspace-surface-audit` to find all live notification sources.
- **Persistence**: Log high-signal events to the MemPalace Diary for historical audit.

## Anti-Patterns
- **The Fan-Out**: Sending every minor comment to both Email and Slack and Desktop.
- **Silent Criticals**: Allowing a failed build on `main` to land in a low-priority digest.
- **Unstructured Noise**: Presenting a raw list of 50 pings without classification or ownership.
