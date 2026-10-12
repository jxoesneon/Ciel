# SLASH_COMMANDS — opencode

OpenCode commands are Markdown files under `~/.config/opencode/commands/`
(global) or `.opencode/commands/` (project). The trimmed body is the prompt
template; frontmatter carries `description`, `agent`, `model`, `subagent`.

Ciel installs four workflow commands (`init/hooks/opencode/install.sh`):

| Command | Purpose | Source |
| --- | --- | --- |
| `/ciel` | Full orchestration skill — route the request and apply mandates | `~/.agents/skills/ciel/SKILL.md` |
| `/ciel-acquire` | Fill a capability gap (registry → MCP → web) | `~/.agents/skills/ciel-acquire/SKILL.md` |
| `/ciel-council` | Council of Five deliberation for high-risk decisions | `~/.agents/skills/ciel-council/SKILL.md` |
| `/ciel-improve` | Self-improvement loop — outcome → verified improvement | `~/.agents/skills/ciel-improve/SKILL.md` |

## Skills (non-slash surface)

OpenCode also auto-discovers the same four as **skills** from
`~/.agents/skills/` (a global compatibility source), so the model can load them
with the `skill` tool even when the user never types a slash command. Commands
are the explicit UX; skills are the model-invoked path.

## Arguments

Templates accept `$ARGUMENTS`, `$1`…`$N`, and `!\`shell\`` blocks. Ciel's
commands intentionally embed no shell blocks (they run outside the tool
permission flow).
