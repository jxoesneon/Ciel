# SPACES — Windsurf Agent Command Center Integration

Windsurf provides **Spaces** — a project-centric organization system in the Agent Command Center that groups agent sessions, PRs, files, and context for a specific task or project.

## Overview

Spaces provide:
- **Project-centric organization** — All related sessions in one view
- **Shared context inheritance** — New sessions inherit Space knowledge
- **Multi-agent coordination** — Local Cascade + cloud Devin sessions together
- **Kanban workflow** — Visual board of agent statuses
- **Persistent state** — View restores exactly as left

## How Spaces Work

### What Lives in a Space

A Space brings together:
- **Agent sessions** — Local Cascade and cloud Devin sessions
- **Pull requests** — PRs opened by you or agents in the Space
- **Files** — Relevant files for the task
- **Context** — Project-level context inherited by new sessions

### Creating a Space

1. **Drag sessions together** — Drag any session onto another in the sidebar
2. **Split pane + new session** — `Cmd/Ctrl+\` then click **New Session**
3. **New tab** — `Cmd/Ctrl+T` opens new session in current Space

### Context Inheritance

When creating a new session in a Space:
- Inherits everything the Space already knows
- No need to re-explain the project
- Agents start working immediately with full context

## Ciel Integration Strategy

### 1. Council of Five as Space-Level Governance

**Concept**: The Council of Five operates at the Space level, providing governance across all sessions within a project.

**Implementation:**

```markdown
# Ciel Space Governance

## Space Initialization
When a Space is created for Ciel-managed project:

1. **Inject Council context** — All sessions inherit Council constitution
2. **Establish risk rubric** — Space-wide risk classification
3. **Configure checkpoints** — Pre-flight checkpoint recommendations
4. **Link to .ciel/ domain** — Space context syncs with `~/.ciel/` and `.ciel/`

## Cross-Session Council Deliberation

When a high-risk request arrives in ANY session within the Space:

1. **Space-wide Council invocation** — All sessions see Council status
2. **Shared deliberation context** — Council reasoning visible across Space
3. **Unified decision** — Single Council decision applies to all sessions
4. **Propagate outcome** — Decision context injected into all Space sessions
```

### 2. Ciel-Managed Space Types

| Space Type | Purpose | Council Role |
| --- | --- | --- |
| **Orchestration** | General skill routing | Light-touch governance |
| **Acquisition** | Skill acquisition workflows | Safety-member approval |
| **Improvement** | Self-improvement loops | Evolution-member + Safety |
| **High-Risk** | Critical system changes | Full Council deliberation |
| **Council Chamber** | Dedicated Council sessions | Chairman-only for voting |

### 3. Space-Based Skill Registry

Ciel can maintain a **Space-local skill registry**:

```markdown
## Space-Local Skills

Skills acquired within a Space can be:

- **Space-private** — Only visible to sessions in this Space
- **Promoted to global** — Synced to `~/.ciel/skills/` after validation
- **Shared with other Spaces** — Via skill export/import
- **Versioned per-Space** — Different Spaces can use different skill versions
```

### 4. Context Persistence Strategy

**Ciel ensures Council context persists across Space sessions:**

```yaml
# Injected into all Space sessions via .windsurf/rules

ciel_space_context:
  space_id: "${SPACE_ID}"
  council_status: "active"
  risk_rubric: "standard"
  checkpoint_policy: "pre_high_risk"
  
  inherited_context:
    - council/CONSTITUTION.md
    - council/members/*.md
    - router/ROUTE_REGISTRY.md
    - risk/RUBRIC.md
    
  session_injection: |
    You are in a Ciel-governed Space.
    Council of Five is active for high-risk decisions.
    Context is shared across all Space sessions.
```

### 5. Multi-Session Workflow Patterns

**Pattern: Parallel Council Deliberation**

```markdown
## Parallel Council in Space

1. **Spawn 5 sessions** — One per Council member in same Space
2. **Share deliberation context** — All see the same case presentation
3. **Parallel reasoning** — Each member responds in their session
4. **Chairman synthesis** — Chairman session aggregates votes
5. **Unified decision** — Outcome propagated to all Space sessions
```

**Pattern: Skill Acquisition Pipeline**

```markdown
## Acquisition Pipeline Space

Sessions in Space:
- **Session 1: Discovery** — Research available implementations
- **Session 2: Sandbox** — Test in isolated environment
- **Session 3: Integration** — Harmonize with existing skills
- **Session 4: Council Review** — Safety + Capability approval
- **Session 5: Documentation** — Update registry + docs

All sessions share:
- Acquisition target
- Test results
- Council decisions
- Final skill artifact
```

## Integration with .ciel/ Domain Model

### Two-Domain + Spaces Model

| Level | Storage | Scope | Persistence |
| --- | --- | --- | --- |
| Global | `~/.ciel/` | Cross-project | Permanent |
| Local | `.ciel/` | Project | Git-tracked |
| Space | `.windsurf/spaces/` | Session group | Space lifetime |

**Ciel sync strategy:**
- **Global** → All Spaces (via `~/.ciel/skills/`)
- **Local** → Spaces in same project (via `.ciel/`)
- **Space** → Ephemeral, can promote to Local/Global

## Implementation: Ciel Space Setup

### Manual Setup (Current)

Users can manually configure Ciel governance in a Space:

1. Create or join a Space
2. Add `.windsurf/rules` with Ciel context:

```markdown
---
description: Ciel Space Governance
globs: "*"
alwaysApply: true
---

# Ciel — Space-Level Orchestration

This Space is governed by Ciel's Council of Five.

## Shared Context
- Constitution: `~/.ciel/council/CONSTITUTION.md`
- Members: `~/.ciel/council/members/`
- Risk Rubric: `~/.ciel/risk/RUBRIC.md`

## Triggers
Activate Council for: high-risk operations, skill acquisition, self-improvement

## Checkpoints
Create named checkpoints before destructive operations.
```

### Future: Automated Space Governance

**If Windsurf exposes Space API:**

```javascript
// Hypothetical Ciel Space Integration
{
  "space_governance": {
    "enabled": true,
    "council": {
      "constitution_path": "~/.ciel/council/CONSTITUTION.md",
      "auto_invoke_high_risk": true
    },
    "checkpoints": {
      "recommend_before_destructive": true,
      "naming_convention": "pre-{operation}-{timestamp}"
    },
    "context_sync": {
      "global_domain": "~/.ciel/",
      "local_domain": "./.ciel/"
    }
  }
}
```

## Cross-Platform Comparison

| Feature | Claude Code | Gemini CLI | Windsurf Spaces |
| --- | --- | --- | --- |
| **Multi-session** | Subagents | Parallel agents | Space sessions |
| **Context sharing** | Parent context | A2A protocol | Space inheritance |
| **Organization** | Session tree | Agent list | Kanban board |
| **Persistence** | Session transcript | Checkpointing | Space restore |
| **Governance scope** | Per-session | Per-agent | Per-Space |

## Best Practices

### For Users

1. **One Space per major task** — Keep context focused
2. **Name Spaces descriptively** — Include project + task
3. **Clean up completed Spaces** — Archive or delete
4. **Use Space for multi-agent workflows** — Parallel sessions work together

### For Ciel

1. **Inject Council context on Space creation** — Establish governance early
2. **Sync with .ciel/ domains** — Maintain two-domain model integrity
3. **Log Space-level decisions** — Activity log tracks cross-session events
4. **Recommend checkpoints at Space level** — Pre-flight safety for whole Space

## Current Status

- **Documented**: Integration patterns and strategies
- **Manual setup**: Users can configure via `.windsurf/rules`
- **Future API**: Pending Windsurf Space API availability

## Related Documents

- `adapters/windsurf/CHECKPOINTS.md` — Named checkpoint integration
- `council/COUNCIL.md` — Council deliberation patterns
- `domains/SPACE_INTEGRATION.md` — Two-domain + Spaces model
