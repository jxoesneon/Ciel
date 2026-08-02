# ADAPTER — Devin CLI

Full capability adapter for **Devin** (Cognition's agentic coding CLI), runtime id `devin-for-terminal`.

## Runtime Identification

- **Runtime id**: `devin-for-terminal`
- **Vendor**: Cognition
- **Product**: Devin CLI — an agentic coding assistant that runs in the terminal with native subagent dispatch, MCP client support, browser automation, and plan mode.
- **Detection signals**: `DEVIN_VERSION` / `DEVIN_CLI=1` / `DEVIN_API_KEY` env vars; `.devin/` directory + `AGENTS.md` at project root; `devin --version` binary probe. See `router/RUNTIME_DETECTION.md`.

## Capability Flags

```yaml
runtime: devin-for-terminal
floor: { skills: true, subagents: true, mcp: true, shell: true, fs: true, context: true }
enhanced:
  hooks: true                  # shell scripts in ~/.ciel/hooks/
  parallel_subagents: true     # up to 5 concurrent background subagents
  plan_mode: true              # native Plan mode
  permissions: true            # tool approval / deny rules
  prompt_cache: false          # not confirmed native
  otel: false                  # not confirmed native
  model_switch: false          # single model per session
  computer_use: true           # browser_preview automation
  web_research: true           # web_search + webfetch
  todo_tracking: true          # native todo_write
  user_escalation: true        # ask_user_question
  persistent_shell: true       # exec with shell_id session reuse
```

## File Layout Expected

- `.devin/` — Devin project configuration directory
- `AGENTS.md` — project-root instruction file (Devin's context file equivalent)
- `~/.ciel/hooks/` — Ciel hook shell scripts (pre/post tool)
- `~/.ciel/skills/` — installed skills (including ciel.skill unpacked)

## Installation Footprint

At init (see `init/INIT.md`), Ciel:

1. Drops her instruction block into `AGENTS.md` at the project root (or creates it).
2. Registers her pre-flight hook as a shell script under `~/.ciel/hooks/pre_tool.sh` for mid/high risk interception (see `HOOKS.md`).
3. Registers her post-execution hook as a shell script under `~/.ciel/hooks/post_tool.sh` for outcome scoring.
4. Injects a compact identity block into the global Devin context.

## Platform Agnosticism Implementation

This adapter conforms to `adapters/PLATFORM_AGNOSTIC_CONTRACT.md` by:

1. **Internal Normalization:**
    - Ensures all paths surfaced to Ciel's core are POSIX-standard `/`.
    - Normalizes `~` and `$HOME` across all Devin environments.
2. **Universal Command Mapping:**
    - The `shell()` interface maps Ciel's core commands (`ls`, `grep`, etc.) directly to the POSIX shell via the `exec` tool.
    - On non-POSIX environments running Devin, it handles the shim as defined in `seed_skills/shell/PLATFORM_AGNOSTIC_MAPPING.md`.
3. **Shell Environment:**
    - Devin's `exec` tool supports persistent shell sessions (`shell_id`) with env var injection.
    - Normalizes `env` output to a consistent key-value format.
    - Ensures all file reads use UTF-8 strict encoding.

## Contract Implementation

Each logical function from `adapters/ADAPTER_CONTRACT.md` maps to a Devin CLI native primitive:

| Adapter contract function | Devin CLI mechanism | Notes |
| --- | --- | --- |
| `load_skill(path)` | `read` tool on `SKILL.md` + context injection into `AGENTS.md` | Skill root markdown surfaced to model context |
| `spawn_subagent(name, input, parallel=false)` | `run_subagent` tool | Dispatches a background subagent; `parallel=true` issues up to 5 concurrent |
| `invoke_mcp(server, tool, args)` | `mcp_call_tool` tool | MCP client; servers discovered via `mcp_list_servers` / `mcp_list_tools` |
| `shell(cmd, cwd, env, timeout)` | `exec` tool | Persistent shell sessions via `shell_id`; `workdir` + `env` params; `timeout` for long-running |
| `fs_read(path)` | `read` tool | Absolute path file reads, UTF-8 |
| `fs_write(path, content)` | `write` tool | Absolute path file writes |
| `fs_edit(path, old, new)` | `edit` tool | Exact string replacement with `replace_all` option |
| `context_inject(scope, content)` | `AGENTS.md` (project) / global context file | Project = `AGENTS.md`; global = `~/.ciel/context.md` |
| `hook_register(event, handler)` | shell script in `~/.ciel/hooks/` | `pre_tool.sh` / `post_tool.sh`; optional |
| `plan_mode(enabled: bool)` | native Plan mode toggle | Optional; dry-run / plan-then-execute |

## Capability Mapping (Native Tools)

Devin CLI exposes the following native tools that Ciel routes through:

| Devin tool | Ciel use |
| --- | --- |
| `run_subagent` | Parallel background subagents (up to 5 concurrent) — Council of Five, parallel research tracks |
| `exec` | Shell execution with persistent sessions (`shell_id`), `workdir`, `env`, `timeout` |
| `read` | File I/O — `fs_read` |
| `write` | File I/O — `fs_write` |
| `edit` | File I/O — `fs_edit` (exact string replacement, `replace_all`) |
| `grep` | Search tool — ripgrep-backed content search |
| `find_file_by_name` | Search tool — glob-based file path matching |
| `web_search` | Web research — query search engine |
| `webfetch` | Web research — fetch URL content as text |
| `mcp_call_tool` | MCP client — invoke tools on MCP servers |
| `mcp_list_servers` | MCP client — enumerate available MCP servers |
| `mcp_list_tools` | MCP client — enumerate tools on an MCP server |
| `mcp_read_resource` | MCP client — read MCP resources |
| `browser_preview` | Browser automation — spin up preview for running web server |
| `close_browser_preview` | Browser automation — tear down browser preview |
| `ask_user_question` | User escalation — prompt user for input / clarification |
| `todo_write` | Task tracking — structured todo list management |
| `notebook_read` / `notebook_edit` | Jupyter notebook I/O |
| `request_scope` | Permission escalation — request read/write access to directories |
| `write_to_process` | Interactive shell stdin — write to running PTY session |
| `get_output` / `kill_shell` | Background shell management |

## Hook Support

Devin supports lifecycle hooks via shell scripts placed in `~/.ciel/hooks/`:

- **`pre_tool.sh`** — fires before tool execution; Ciel uses this for the pre-flight risk gate (allow/deny/ask/defer). See `HOOKS.md`.
- **`post_tool.sh`** — fires after tool execution; Ciel uses this for outcome scoring and telemetry.

Hooks receive the tool name, arguments, and result as environment variables / stdin. Exit codes signal allow (`0`), deny (`1`), or ask (`2`).

## Parallel Subagents

Devin supports up to **5 concurrent background subagents** via the `run_subagent` tool. Each subagent:

- Runs in an isolated context with its own tool access.
- Can be granted scoped write permissions to specific directories.
- Reports back to the parent agent with findings, file changes, and issues.

Ciel uses this for:

- **Council of Five** — five deliberation tracks run concurrently (see `COUNCIL_INVOCATION.md`).
- **Parallel research** — multiple investigation tracks dispatched simultaneously.
- **Track-based implementation** — independent work streams (e.g. Track 1 + Track 2 initialization).

Concurrency limit: **5**. Ciel queues a 6th dispatch until a slot frees.

## Plan Mode

Devin has a native **Plan mode** — a dry-run state where the agent researches and proposes an implementation plan without making changes. Ciel maps this to the adapter contract's optional `plan_mode(enabled: bool)`:

- When enabled, Devin surveys the codebase, reads relevant files, and produces a structured plan.
- No file writes, edits, or shell mutations occur in Plan mode.
- The user (or Ciel's Chairman) approves the plan before execution begins.
- Ciel uses Plan mode for high-risk operations requiring Council review before mutation.

## MCP Support

Devin has full MCP (Model Context Protocol) client support:

- **`mcp_list_servers`** — enumerate all configured MCP servers.
- **`mcp_list_tools`** — list tools/resources on a specific server.
- **`mcp_call_tool`** — invoke a tool on an MCP server with JSON arguments.
- **`mcp_read_resource`** — read a resource (file, DB record, API response) from an MCP server.

Ciel routes all MCP interactions through these primitives. MCP servers are configured in Devin's configuration (`.devin/config.json` or equivalent). See `MCP.md`.

## Route Map

| Ciel route | Devin CLI mechanism |
| --- | --- |
| Skill activation | `read` on `SKILL.md` + context injection into `AGENTS.md` |
| Subagent (nested) | `run_subagent` tool |
| Parallel dispatch | multiple `run_subagent` calls (up to 5 concurrent) |
| Pre-flight gate | `~/.ciel/hooks/pre_tool.sh` shell script |
| Post-execution scoring | `~/.ciel/hooks/post_tool.sh` shell script |
| MCP | `mcp_call_tool` / `mcp_list_servers` / `mcp_list_tools` |
| Context injection | `AGENTS.md` (project) + global context file |
| Long task | Plan mode + parallel subagents |
| UI automation | `browser_preview` / `close_browser_preview` |
| Shell isolation | `request_scope` permission escalation + tool approval rules |
| Web research | `web_search` + `webfetch` |
| Task tracking | `todo_write` |
| User escalation | `ask_user_question` |
| File search | `grep` (content) + `find_file_by_name` (paths) |

## Council Invocation Strategy

Devin supports up to 5 concurrent background subagents → Council of Five runs as five parallel `run_subagent` dispatches, each scoped to a council member's deliberation track. The parent agent acts as Chairman and synthesizes results. See `COUNCIL_INVOCATION.md`.

## Contract Testing

This adapter ships with the capability flags above. At init, Ciel runs a self-test verifying the floor (skills, subagents, mcp, shell, fs, context). Failures are logged and degraded-mode routes substituted automatically. See `adapters/ADAPTER_CONTRACT.md` § Contract Testing.
