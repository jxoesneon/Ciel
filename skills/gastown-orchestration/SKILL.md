---
name: gastown-orchestration
version: 1.0.0
format: skill/1.0
description: CIEL's framework for CLI-toolchain multi-agent orchestration via the gt/bd command suite. Manages agent lifecycles, work slinging, and crash recovery.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
side_effects: ["shell", "state_mutation"]
triggers:
  - pattern: "(gastown|gas town|gt command|bd command|sling work|polecat|convoy|rig|bead)"
    confidence: 0.9
  - pattern: "(crew|refinery|witness|mayor|deacon|GUPP|hook|molecule)"
    confidence: 0.8
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: ["gt", "bd"] }
---

# CIEL ADAPTATION: Gas Town Orchestration (CLI Toolchain)

This skill adapts the Gas Town CLI toolchain (`gt`/`bd`) into Ciel's vocabulary. It provides a CLI-driven alternative to Ciel's internal fleet model — where `autonomous-orchestration` uses Ciel-native worktrees and missions, gastown uses external process spawning, bead-based work tracking, and mail-triggered patrols. Ciel's risk classifier gates subprocess spawning as mid→LLM_JUDGE or high→Council.

## Core Vocabulary (Ciel Translation)

- **Bead** (`bd`): A tracked work unit. Ciel equivalent: a mission task ticket.
- **Convoy**: A batch of related beads. Ciel equivalent: a mission DAG node group.
- **Rig**: A registered project workspace (GitHub repo). Ciel equivalent: an isolated worktree target.
- **Polecat**: Ephemeral worker spawned per-task, vanishes on completion. Ciel equivalent: a one-shot sub-agent.
- **Crew**: Persistent named worker with ongoing sessions. Ciel equivalent: a long-lived fleet agent.
- **Hook**: Where work lands for a worker. **GUPP** (Gas Town Universal Propulsion Principle): if hook has work, run it.
- **Molecule**: A workflow unit that survives crashes — any worker can resume where another left off.
- **Patrols**: Witness (watches for stuck workers), Refinery (merges completed work), Mayor (coordinates rigs).

## CLI Command Patterns

**Engine Control**: `gt up` (start), `gt down` (graceful stop), `gt status` (overview)
**Work Management**: `gt sling <bead> <rig>` (assign work), `gt convoy list`, `gt hook`
**Workers**: `gt polecat list`, `gt crew list`, `gt peek <agent>`, `gt nudge <agent> "msg"`
**Diagnostics**: `gt doctor` (health check), `gt doctor --fix` (auto-repair), `bd doctor`, `gt feed` (activity stream)
**Beads**: `bd list`, `bd show <id>`, `bd sync`, `bd create --title "..."`
**Refinery**: `gt refinery start`, `gt refinery status`, `gt refinery queue`
**Patrol Activation**: `gt mail send <rig>/witness -s "Patrol" -m "Process completed work"`

## Lifecycle Management

1. **Install**: `go install .../gt@latest` + `go install .../bd@latest`; verify with `gt doctor` AND `bd doctor`.
2. **Add Rig**: Register a project; verify patrols exist via `gt doctor --fix`.
3. **Create Work**: `bd create --title "..."` → bead ID assigned.
4. **Sling**: `gt sling <bead> <rig>` → polecat spawns, work lands on hook, GUPP executes.
5. **Monitor**: `gt peek <agent>`, `gt feed`; Witness patrols detect stuck workers.
6. **Merge**: Refinery processes completed work → code lands on main.
7. **Shutdown**: `gt down` for graceful stop.

## Crash Recovery

- Molecules survive crashes — any worker resumes another's in-progress molecule.
- Run `gt doctor --fix` to repair prefix mismatches, missing patrols, daemon issues.
- `bd sync` restores bead consistency across clones.
- If a polecat is stuck: `gt nudge <agent> "msg"` → trigger Witness patrol → if still stuck, pull work and `gt polecat nuke`.

## Verification Protocol

Never declare "ready" without full-flow verification: create test bead → sling → polecat completes → Witness marks ready → Refinery processes → code on main. If ANY step fails, investigate before proceeding.

## Anti-Patterns

- **Manual Agent Beads**: Creating agent beads by hand — `gt sling` does this automatically.
- **Guessing Session Names**: Always use `gt polecat list` to get actual names.
- **Assuming Patrols Self-Activate**: Witness/Refinery are agents, not daemons — send mail to trigger them.
- **Partial Readiness**: Declaring the system working after `gt doctor` passes but before testing the full sling→merge flow.
