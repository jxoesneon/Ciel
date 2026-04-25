---
name: autonomous-orchestration
version: 1.0.0
format: skill/1.0
description: CIEL's framework for multi-agent orchestration, crons, and isolated worktree execution.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(orchestrate|dispatch|schedule).*(agent|fleet|cron|mission)"
    confidence: 0.95
  - pattern: "multi-agent workflow"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Autonomous Orchestration (Fleets & Missions)

This skill formalizes the execution of multi-step, multi-agent workflows. it prioritizes isolation, structured reporting, and persistent scheduling.

## The Fleet Pattern (DevFleet)
1. **Mission DAG**: Break complex requests into a Directed Acyclic Graph (DAG) of missions.
2. **Isolation**: Each agent MUST run in an isolated git worktree with dedicated tooling.
3. **Auto-Dispatch**: Missions with met dependencies auto-start. missions with blocked dependencies queue.
4. **Merge Protocol**: Missions auto-merge on success; conflict resolution is a manual or dedicated "Conflict Solver" mission.

## Scheduled Operations (Crons)
- **Persistence**: Use native scheduled tasks for recurring monitoring (e.g., Daily standup at 9 AM).
- **Session Isolation**: Crons run in isolated sessions. use project memory for cross-session context.
- **Error Handling**: Every cron prompt MUST include instructions for handling failures (e.g., "If tests fail, post log to memory and alert user").

## Reporting & Status
- **Dashboard**: Use `get_dashboard()` to monitor slot usage and recent activity.
- **Missions**: Every mission MUST produce a `MISSION_REPORT.md` (Files changed, Tests run, Next steps).

## Anti-Patterns
- **Circular Missions**: Creating a dependency loop between agents.
- **Shared Worktrees**: Running two independent agents in the same directory (causes race conditions).
- **Amnesiac Crons**: Running a daily task that doesn't read the previous day's result from memory.
