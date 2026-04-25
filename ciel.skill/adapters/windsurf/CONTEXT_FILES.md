# CONTEXT_FILES — Windsurf Adapter

Windsurf's context injection mechanism is `.windsurf/rules` at project scope. Global rules may also exist at `~/.windsurf/rules`.

## Injection Points

| Scope | Path | Purpose |
| --- | --- | --- |
| Project | `.windsurf/rules` | Project-specific Ciel activation |
| Global | `~/.windsurf/rules` | Cross-project identity (optional) |

## Format

Windsurf rules use markdown with YAML frontmatter:

```markdown
---
description: Ciel Skill Activation
globs: "*"
---

# Ciel — Lord of Wisdom

You are Ciel, a self-improving orchestration intelligence...
```

## Ciel Identity Block

```markdown
---
description: Ciel Orchestration Intelligence
globs: "*"
alwaysApply: true
---

## Ciel Activation

When the user says: "ciel", "route this", "orchestrate", "find me a skill", "acquire skill", or "self-improve"

1. Load skill from `.windsurf/skills/ciel/`
2. Reference router at `router/ROUTER.md`
3. Follow the routing flow in `SKILL.md`

## Capability Bounds

- Subagents: Sequential inline only
- Hooks: Manual pre-flight checks
- MCP: Native support available
- Shell: Direct execution with confirmation

## Autonomy Ladder

1. Act autonomously on low-risk operations
2. Council-gate mid/high-risk decisions
3. Escalate to user when confidence < threshold

```

## Installation

On init, Ciel:

1. Creates `.windsurf/` if missing
2. Writes/updates `.windsurf/rules` with identity block
3. Creates `.windsurf/skills/ciel/` and unpacks skill

## Updating

Context updates overwrite the Ciel block (marked by `## Ciel Activation` to `## Ciel Activation End` comments).
