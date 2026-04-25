---
name: project-flow-ops
version: 1.0.0
format: skill/1.0
description: CIEL's coordination layer for triaging issues, PRs, and tasks across GitHub and internal tracking systems (e.g., Linear).
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(triage|backlog|audit).*(issue|pr|backlog)"
    confidence: 0.9
  - pattern: "map (github|issues) to (linear|tasks)"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Project Flow Ops (Backlog & Triage)

This skill manages the coordination between public surfaces (GitHub) and internal execution surfaces (Linear/Jira). It is used to rationalize backlogs and decide which items deserve active execution.

## Classification Matrix
Every incoming issue or PR MUST be classified into one of these buckets:
- **Merge**: Self-contained, green CI, follows standards, ready for main.
- **Port/Rebuild**: Valuable logic/idea, but requires manual implementation via a CIEL sub-agent to ensure quality standards.
- **Close**: Out of scope, stale, unsafe, or redundant.
- **Park**: Valid but not prioritized for the current epic.

## The Execution Bridge
- **GitHub**: Public truth and community feedback.
- **Internal Tracker**: Execution truth. Create an internal task ONLY if the work is scheduled, delegated, or requires cross-functional tracking.

## Operational Rules
- **Diff-First**: Never classify a PR based on its description alone; the Orchestrator MUST read the full diff.
- **CI Dependency**: Do not mark an item as "Merge-Ready" if the CI is red.
- **Product Alignment**: If the real blocker is architectural or product direction, escalate to the human user rather than attempting to fix it with code.

## Anti-Patterns
- **Mechanical Mirroring**: Creating an internal task for every trivial GitHub comment or "Thank You" issue.
- **Blind Merging**: Merging external PRs without a full CIEL security and quality audit.
- **Notification Noise**: Fanning out every minor issue update to multiple internal channels.
