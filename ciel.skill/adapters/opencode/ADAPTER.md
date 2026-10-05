# ADAPTER — opencode

Full capability adapter for **opencode** (anomalyco/opencode TUI/CLI), runtime id `opencode`.

## Runtime Identification

- **Runtime id**: `opencode`
- **Vendor**: anomalyco
- **Product**: opencode — terminal coding agent with MCP client, agents, plugins, and persistent sessions.
- **Detection signals**: `~/.config/opencode/opencode.json(c)` present; `OPENCODE_` env vars; `opencode` binary probe. See `router/RUNTIME_DETECTION.md`.

## Capability Flags

```yaml
runtime: opencode
floor: { skills: true, subagents: true, mcp: true, shell: true, fs: true, context: true }
enhanced:
  hooks: true                  # opencode plugin API (plugins/*.ts)
  parallel_subagents: true     # Task tool subagents
  plan_mode: true              # plan agent permission ruleset
  permissions: true            # opencode.json permission config
  computer_use: true           # via ultranix-mcp MCP server
  web_research: true           # webfetch / websearch tools
  todo_tracking: true          # todowrite tool
  user_escalation: true        # question tool
  prompt_cache: false
  otel: true                   # opencode.log tracing
```

## File Layout Expected

- `~/.config/opencode/opencode.jsonc` — global config (mcp, plugin, permission, instructions)
- `~/.config/opencode/plugins/*.ts` — auto-loaded plugins (ciel.ts hooks bridge)
- `~/.config/opencode/AGENTS.md` — global instruction file (Ciel identity)
- `~/.config/opencode/tui.json` — TUI plugin/theme
- `.opencode/opencode.json(c)` — project-level config (deep-merged)
- Project `AGENTS.md` — project instructions

## Installation Footprint

1. `~/.config/opencode/AGENTS.md` — Ciel identity block.
2. `~/.config/opencode/plugins/ciel.ts` — hooks bridge (session-start, prompt-submit, pretool, post-tool).
3. `opencode.jsonc` `plugin` entry registers `./plugins/ciel.ts`.
4. MCP servers registered under `mcp:` (github, martensite, ultranix).

## Contract Implementation

| Adapter contract function | opencode mechanism | Notes |
| --- | --- | --- |
| `load_skill(path)` | `skill` tool + `skills.paths` config | Skills in `~/.agents/skills/` auto-loaded |
| `spawn_subagent(name, input, parallel=false)` | `task` tool (explore/general agents) | Up to N concurrent Task calls |
| `invoke_mcp(server, tool, args)` | MCP client | Servers in `opencode.jsonc` `mcp:` |
| `shell(cmd, cwd, env, timeout)` | `bash` tool | Persistent session per process spawn |
| `fs_read/write/edit` | `read` / `write` / `edit` tools | Permission-gated |
| `context_inject(scope, content)` | `AGENTS.md` + plugin chat.message transform | Global AGENTS.md read at startup |
| `hook_register(event, handler)` | plugin hooks (`event`, `chat.message`, `tool.execute.before/after`, `experimental.chat.messages.transform`) | See `HOOKS.md` |
| `plan_mode(enabled)` | plan agent (`permission.edit: deny *`) | toggled per-agent |

## Hook Support

Hooks live in the opencode plugin (`plugins/ciel.ts`) — NOT shell scripts in `~/.ciel/hooks/opencode/`. Mapping:

| Ciel hook | opencode plugin hook | Payload |
| --- | --- | --- |
| session_start | `event` → `session.created` | runs `ciel session-start --runtime devin`, injects additionalContext on first transform |
| user_prompt_submit | `chat.message` | `ciel prompt-submit` scans raw text, appends warning on secret hits |
| pre_tool_use | `tool.execute.before` | `ciel pretool --runtime devin` with `{tool_name, tool_input}`; `decision: block` aborts |
| post_tool_use | `tool.execute.after` | `ciel post-tool --runtime devin` with `{tool_name, tool_response}` |

`--runtime devin` is used because the ciel binary does not yet have an opencode runtime table; opencode hook JSON shape is compatible. Upstream task: add `opencode` to ciel-rs runtime match arms.

## MCP Support

Full MCP client. Configured servers in `opencode.jsonc` under `mcp`:

- `github` — github-mcp-server (issues, PRs, code search) via `{file:}` token
- `martensite` — `cargo-martensite mcp` (design lint, widget inspect, tweaks, scaffold)
- `ultranix` — desktop automation vision/input/clipboard

See `MCP.md`.

## Route Map

| Ciel route | opencode mechanism |
| --- | --- |
| Skill activation | `skill` tool lookup |
| Subagent | `task` tool (parallel calls allowed) |
| Pre-flight gate | plugin `tool.execute.before` → ciel pretool |
| Post-execution scoring | plugin `tool.execute.after` → ciel post-tool |
| MCP | MCP client, servers from config |
| Context injection | global + project AGENTS.md, plugin message transform |
| UI automation | ultranix-mcp tools |
| Computer use | ultranix-mcp vision/input categories |
| Web research | webfetch + websearch tools |
| Task tracking | todowrite tool |
| User escalation | question tool |

## Council Invocation Strategy

Parallel-top-level via concurrent `task` tool dispatches for each Council member persona, parent agent as Chairman. See `COUNCIL_INVOCATION.md`.

## Known Gaps / Upstream Tasks

1. ciel-rs `--runtime` table lacks `opencode` → plugin passes `devin` payloads (compatible shapes).
2. opencode plugin API is not stable across versions — pin `$schema` and test on upgrade.
3. `experimental.chat.messages.transform` is experimental — canary injection may need moving to `chat.params`/`chat.headers` hooks on API drift.
