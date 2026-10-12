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

Deployed by `init/hooks/opencode/install.sh`:

1. `~/.config/opencode/plugins/ciel.ts` — hooks bridge (`session-start`,
   `prompt-submit`, `pretool` gate, `post-tool`, `session-end`). Auto-discovered;
   **no** `plugins` entry in `opencode.jsonc` is required.
2. `~/.config/opencode/AGENTS.md` — Ciel identity anchor block.
3. `~/.config/opencode/commands/ciel*.md` — workflow commands (see `WORKFLOWS.md`).
4. Skills auto-discovered from `~/.agents/skills/ciel*` (global compatibility source).
5. MCP servers registered under `mcp:` in `opencode.jsonc` (see `MCP.md`).

## Contract Implementation

| Adapter contract function | opencode mechanism | Notes |
| --- | --- | --- |
| `load_skill(path)` | `skill` tool + `skills.paths` config | Skills in `~/.agents/skills/` auto-loaded |
| `spawn_subagent(name, input, parallel=false)` | `task` tool (explore/general agents) | Up to N concurrent Task calls |
| `invoke_mcp(server, tool, args)` | MCP client | Servers in `opencode.jsonc` `mcp:` |
| `shell(cmd, cwd, env, timeout)` | `bash` tool | Persistent session per process spawn |
| `fs_read/write/edit` | `read` / `write` / `edit` tools | Permission-gated |
| `context_inject(scope, content)` | `AGENTS.md` + plugin chat.message transform | Global AGENTS.md read at startup |
| `hook_register(event, handler)` | V2 domain hooks: `ctx.session.hook("prompt"\|"context")`, `ctx.tool.hook("execute.before"\|"execute.after")`, `ctx.permission.hook("evaluate")`, `ctx.event.subscribe` | See `HOOKS.md` |
| `plan_mode(enabled)` | plan agent (`permission.edit: deny *`) | toggled per-agent |

## Hook Support

Hooks live in the opencode plugin (`plugins/ciel.ts`) — **not** shell scripts in
`~/.ciel/hooks/opencode/`. The `ciel` binary supports `--runtime opencode`
natively for every shared hook body — `pretool`, `prompt-submit`,
`session-start`, `post-tool`, `session-end`, and `stop`. Mapping:

| Ciel hook | opencode V2 hook | Payload / effect |
| --- | --- | --- |
| session_start | `setup(ctx)` (once) | `ciel session-start --runtime opencode` → **AI canary** captured |
| context inject | `ctx.session.hook("context")` | canary prepended to `event.system[]` on every model request |
| user_prompt_submit | `ctx.session.hook("prompt")` | `ciel prompt-submit --runtime opencode`; appends secret-scan warning to `event.prompt.text` |
| pre_tool_use | `ctx.tool.hook("execute.before")` | `ciel pretool --runtime opencode` → throws on `decision: block` |
| post_tool_use | `ctx.tool.hook("execute.after")` | `ciel post-tool --runtime opencode` |
| session_end | `ctx.event.subscribe(...)` | `ciel session-end --runtime opencode` |

Full detail, payload shapes, and the fallback contract: `HOOKS.md`.

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

1. The `ciel-rs` `opencode` runtime reuses the Devin field extraction (no
   opencode-specific arm). Field names are compatible, so this is harmless; the
   shared hook bodies now honor `--runtime` for audit labeling (`prompt-submit`,
   `stop`). A runtime-tailored canary string remains a future nicety.
2. The gate uses `ctx.tool.hook("execute.before")` (throw aborts). The
   documented `ctx.permission.hook("evaluate")` deny path is an equivalent
   alternative but double-writes the activity log, so it is not used.
3. The V2 plugin API is not frozen — re-run `init/hooks/opencode/install.sh` and
   smoke-test after an OpenCode upgrade.

## Related

- `HOOKS.md` — hook mapping + payload shapes + verification
- `CONTEXT_FILES.md` — global `AGENTS.md` identity anchor
- `SUBAGENTS.md` — task-tool delegation and guilds
- `SLASH_COMMANDS.md` — `/ciel*` commands
- `WORKFLOWS.md` — acquire / council / improve surfaces
- `MCP.md` — MCP servers and discovery
- `COUNCIL_INVOCATION.md` — Council of Five topology
