# WORKFLOWS — opencode

Ciel workflows are host-agnostic; OpenCode surfaces them three ways.

| Surface | Path | Invocation |
| --- | --- | --- |
| Skills | `~/.agents/skills/ciel-*/SKILL.md` | `skill` tool, or `@ciel-acquire` |
| Commands | `~/.config/opencode/commands/ciel-*.md` | `/ciel-acquire`, `/ciel-council`, `/ciel-improve` |
| Agent personas | `.opencode/agent/*.md`, `~/.config/opencode/agent/*.md` | `--agent <id>` / agent mentions |

## Canonical workflows

1. **acquire** — capability gap → registry → MCP → web research → harmonize.
   Source: `~/.agents/skills/ciel-acquire/SKILL.md`.
2. **council** — high-risk decision → five parallel members (Stage 1) →
   anonymized cross-review (Stage 2) → Chairman synthesis + docket.
   Source: `~/.agents/skills/ciel-council/SKILL.md`, `COUNCIL_INVOCATION.md`.
3. **improve** — interaction outcome → score → verified improvement.
   Source: `~/.agents/skills/ciel-improve/SKILL.md`.

## Triggering

- **Explicit**: the `/ciel-*` commands, or `@skill-id` mentions.
- **Automatic**: the router in `router/RUNTIME_DETECTION.md` fingerprints the
  request and loads the matching workflow when the trigger conditions in each
  `SKILL.md` frontmatter are met.

## Verification

```sh
ls ~/.config/opencode/commands/ciel*.md
ls ~/.agents/skills/ciel*/SKILL.md
```
