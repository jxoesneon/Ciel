# ARTIFACTS

CIEL 1.0 utilizes an isolated artifact system to ensure the codebase remains high-signal and free of transient reasoning data.

## Lifecycle
1. **Generation**: Created during 'Research' or 'Strategy' phases.
2. **Review**: Gated by the Council or HITL Protocol for high-risk operations.
3. **Verification**: Referenced during 'Execution' as proof-of-work.
4. **Archival**: Automatically moved to `~/.ciel/artifacts/` upon task completion.

## Structure
- `plans/`: Implementation blueprints and task lists.
- `audits/`: Independent and subagent review reports.
- `visuals/`: Evidence-based screenshots and terminal traces.
- `transient/`: Ephemeral data and context-management swap files.

---
**Status**: Active. **Isolation**: Enforced.
