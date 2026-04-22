# ADAPTER — Claude Code

Full capability adapter for Claude Code (Anthropic's agent CLI).

## Capability Flags

```yaml
runtime: claude-code
floor: { skills: true, subagents: true, mcp: true, shell: true, fs: true, context: true }
enhanced:
  hooks: true                  # PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, PermissionDenied
  parallel_subagents: true     # via multiple Agent tool calls
  plan_mode: false             # not native (handled by permission deny pattern)
  permissions: true            # allowedTools / deny rules
  prompt_cache: true           # 1h TTL
  otel: true                   # native
  model_switch: true           # haiku/sonnet/opus mid-session
  computer_use: preview
  ultraplan: true
  remote_control: true
```

## File Layout Expected

- `.claude/agents/` — subagent markdowns
- `.claude/commands/` — slash commands
- `.claude/settings.json` — permissions + hooks
- `CLAUDE.md` / `~/.claude/CLAUDE.md` — context hierarchy
- `~/.claude/skills/` — installed skills (including ciel.skill unpacked)

## Installation Footprint

At init (see `init/INIT.md`), Ciel:

1. Drops her slash commands under `.claude/commands/` (`/ciel`, `/ciel-init`, `/ciel-council`).
2. Registers her pre-flight hook at `PreToolUse` for mid/high risk interception (see `HOOKS.md`).
3. Registers her post-execution hook at `PostToolUse` for outcome scoring.
4. Injects a compact identity block into `CLAUDE.md` (or creates it).

## Platform Agnosticism Implementation

This adapter conforms to `adapters/PLATFORM_AGNOSTIC_CONTRACT.md` by:

1. **Internal Normalization:**
    - Ensures all paths surfaced to Ciel's core are POSIX-standard `/`.
    - Normalizes `~` and `$HOME` across all Claude Code environments.
2. **Universal Command Mapping:**
    - The `shell()` interface maps Ciel's core commands (`ls`, `grep`, etc.) directly to the POSIX shell.
    - On non-POSIX environments running Claude Code, it handles the shim as defined in `seed_skills/shell/PLATFORM_AGNOSTIC_MAPPING.md`.
3. **Shell Environment:**
    - Normalizes `env` output to a consistent key-value format.
    - Ensures all file reads use UTF-8 strict encoding.

## Route Map

| Ciel route | Claude Code mechanism |
| --- | --- |
| Skill activation | `/skills` or auto-load via `SKILL.md` header |
| Subagent (nested) | `Agent` tool → `.claude/agents/<name>.md` |
| Parallel dispatch | multiple `Agent` tool calls in one turn |
| Pre-flight gate | `PreToolUse` hook allow/deny/ask/defer |
| Post-execution scoring | `PostToolUse` + `PostToolUseFailure` |
| MCP | `ToolSearch` lazy schema loading |
| Context injection | `CLAUDE.md` hierarchy |
| Long task | Ultraplan + cloud env |
| UI automation | Computer use (preview-aware) |
| Shell isolation | `allowedTools` / deny rules |
| Headless script | `claude -p --output-format stream-json` |

See the per-file specs: `HOOKS.md`, `SUBAGENTS.md`, `SLASH_COMMANDS.md`, `MCP.md`, `COMPUTER_USE.md`, `ULTRAPLAN.md`, `PROMPT_CACHE.md`, `PERMISSIONS.md`, `REMOTE_CONTROL.md`, `COUNCIL_INVOCATION.md`, `CONTEXT_FILES.md`.

## Council Invocation Strategy

Claude Code supports nested subagent trees → Council of Five runs as a nested deliberation with a parent Chairman agent. See `COUNCIL_INVOCATION.md`.
