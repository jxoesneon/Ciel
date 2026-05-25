# ADAPTER — Devin for Terminal

Full capability adapter for Devin for Terminal (Cognition's CLI agent, bundled with Windsurf and standalone).

## Capability Flags

```yaml
runtime: devin-for-terminal
floor: { skills: true, subagents: true, mcp: true, shell: true, fs: true, context: true }
enhanced:
  hooks: true                  # PreToolUse, PostToolUse, PermissionRequest, UserPromptSubmit, SessionStart, SessionEnd, Stop, PostCompaction
  parallel_subagents: true   # background subagents with profiles
  plan_mode: true              # /plan slash command
  permissions: true          # scope-based allow/deny/ask (Read, Write, Exec, Fetch) + tool-based
  prompt_cache: false        # not yet documented
  otel: false                # not yet documented
  model_switch: true         # /model slash command
  computer_use: false        # not available in terminal
  sandbox: true              # --sandbox with OS-level isolation
  modes: true                # Normal, Accept Edits, Bypass, Autonomous, Plan
  custom_subagents: true     # .devin/agents/<name>/AGENT.md profiles
  skill_paths: true          # .devin/skills/ + ~/.config/devin/skills/
  slash_commands: true       # /hooks, /mode, /plan, /bypass, /accept-edits, /normal, /loop, /workspace
```

## File Layout Expected

- `.devin/config.json` — project permissions, MCP servers, hooks, read_config_from
- `.devin/config.local.json` — personal overrides (gitignored)
- `.devin/hooks.v1.json` — standalone hooks file (Claude Code compatible format)
- `.devin/skills/` — project-specific skills
- `.devin/agents/` — custom subagent profiles (`AGENT.md` format)
- `AGENTS.md` / `AGENT.md` / `CLAUDE.md` — always-on rules (read automatically)
- `~/.config/devin/config.json` — user-wide settings, global skills, global hooks
- `~/.config/devin/skills/` — global skills
- `~/.config/devin/agents/` — global custom subagents

## Cross-Runtime Compatibility

Devin reads `.claude/` configuration by default (`read_config_from.claude: true`), meaning existing Claude Code hooks, commands, and skills work transparently. Ciel's `.claude/settings.json` hooks are therefore **functional on Devin**.

However, the **native Devin format** (`.devin/hooks.v1.json` and `.devin/config.json`) is preferred for:
- Cleaner project structure (everything in `.devin/`)
- Avoiding collisions if the project is also used with Claude Code directly
- Leveraging Devin-specific features (scope-based permissions, sandbox config)

## Installation Footprint

At init, Ciel on Devin:

1. Creates `.devin/hooks.v1.json` with Ciel lifecycle hooks (if not present).
2. Creates `.devin/config.json` with project permissions and `read_config_from` settings.
3. Drops `.devin/skills/ciel/` symlink or copy (project-scoped skill).
4. Optionally creates `.devin/agents/ciel/AGENT.md` for Council of Five subagent profile.
5. Injects a compact identity block into `AGENTS.md` (or creates it).
6. Updates `~/.config/devin/config.json` with global `SessionStart` hook.

## Route Map

| Ciel route | Devin mechanism |
| --- | --- |
| Skill activation | `/skills` slash command or auto-load via `SKILL.md` header |
| Subagent (nested) | `run_subagent` tool → `.devin/agents/<name>/AGENT.md` |
| Parallel dispatch | background `run_subagent` with `is_background: true` |
| Pre-flight gate | `PreToolUse` hook → `decision: approve/block/ask` |
| Post-execution scoring | `PostToolUse` hook |
| MCP | `mcp_call_tool` / `mcp_read_resource` via config |
| Context injection | `AGENTS.md` / `CLAUDE.md` hierarchy + `.windsurf/rules` |
| Plan mode | `/plan` slash command or `--permission-mode plan` |
| Mode switching | `/normal`, `/accept-edits`, `/bypass`, `/autonomous` |
| Sandbox | `devin --sandbox --permission-mode autonomous` |
| Loop automation | `/loop <prompt>` for review loops |
| Workspace mgmt | `/workspace`, `/add-dir <path>` |
| Hook verification | `/hooks` slash command |

## Permission Model

Devin uses **scope-based** permissions (distinct from Claude Code's tool-based approach):

```json
{
  "permissions": {
    "allow": ["Read(src/**)", "Exec(git)", "Exec(npm run)"],
    "deny": ["Exec(rm)", "Exec(sudo)", "Write(.env*)"],
    "ask": ["Write(**/.env*)"]
  }
}
```

Ciel's Safety member maps its risk classifications to Devin permission scopes:
- `critical` → `deny` rule + block hook
- `high` → `ask` rule + ask hook
- `mid` → `ask` rule (per-project configurable)
- `low` → `allow` rule + approve hook

## Subagent Profiles

Devin supports custom subagent profiles via `AGENT.md` files. Ciel's Council of Five can be implemented as:

- `.devin/agents/ciel-chairman/AGENT.md` — deliberation orchestrator
- `.devin/agents/ciel-safety/AGENT.md` — risk veto member
- `.devin/agents/ciel-efficiency/AGENT.md` — bloat/perf member
- `.devin/agents/ciel-coherence/AGENT.md` — consistency member
- `.devin/agents/ciel-evidence/AGENT.md` — research validation member

See `SUBAGENTS.md` for full profile specifications.

## Devin-Specific Hooks

Devin hooks use the **Claude Code format** (JSON in `.devin/hooks.v1.json` or `.devin/config.json` `"hooks"` key).

Supported events: `PreToolUse`, `PostToolUse`, `PermissionRequest`, `UserPromptSubmit`, `Stop`, `SessionStart`, `SessionEnd`, `PostCompaction`

Hook output: `{ "decision": "approve" | "block" | "ask" }` (exit code 2 also blocks)

See `HOOKS.md` for Ciel's full Devin hook configuration.
