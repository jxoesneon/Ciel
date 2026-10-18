# ADAPTER — Freebuff

Full-capability adapter for **Freebuff** (freebuff.com — the free agentic coding runtime), runtime id `freebuff`.

## Runtime Identification

- **Runtime id**: `freebuff`
- **Vendor**: Freebuff (Codebuff lineage — same runtime family, `CODEBUFF_*` env markers retained)
- **Product**: A terminal/IDE-agnostic agentic coding agent ("Buffy") that runs where its CLI binary runs, with a `skill` tool for SKILL.md loading, ripgrep-backed code search, structured file editing, background shell sessions, and user escalation prompts.
- **Detection signals**: `FREEBUFF_*` or `CODEBUFF_*` env vars (`CODEBUFF_TREE_SITTER_WASM_PATH`, `CODEBUFF_RG_PATH` → `/usr/local/lib/freebuff/`); `.freebuff/` project directory containing `project-id`; `freebuff` binary (`~/.local/bin/freebuff` wrapper or `/usr/local/bin/freebuff`); `freebuff --version` probe. See `router/RUNTIME_DETECTION.md`.

## Capability Flags

```yaml
runtime: freebuff
floor: { skills: true, subagents: degraded, mcp: false, shell: true, fs: true, context: true }
enhanced:
  hooks: false                 # no lifecycle-hook registration; risk gate is inlined per tool call
  parallel_subagents: false    # council / research tracks run sequentially (see COUNCIL_INVOCATION.md)
  plan_mode: false             # soft equivalent: write_todos plan + describe-before-execute
  permissions: true            # ask_user approval gates; scoped, task-bounded authorization
  prompt_cache: false          # not confirmed native
  otel: false                  # not confirmed native
  model_switch: false          # model is fixed per session by the host
  computer_use: false          # no browser automation surface
  web_research: true           # web_search + read_url
  todo_tracking: true          # write_todos
  user_escalation: true        # ask_user (single/multi-select), suggest_followups post-task
  background_shell: true       # run_terminal_command BACKGROUND + log inspection
  structured_edit: true        # str_replace exact-match editing, write_file full-content writes
  search: true                 # code_search (ripgrep, regex, flags) + glob + list_directory
```

`subagents: degraded` means there is no native spawn-subagent tool in the current runtime surface; delegation primitives are **markdown-file-defined personas** loaded via the `skill` tool and deliberated in-context, serially. This satisfies the floor's "markdown-file-defined delegation primitive (nested)" clause. If a session exposes an agent-spawning tool, upgrade the flag and route parallel dispatch through it.

## File Layout Expected

- `.freebuff/` — per-project runtime directory (`project-id`)
- `AGENTS.md` — project-root context file (Freebuff's persistent instruction file; hierarchical AGENTS.md files in subdirectories are also honored)
- `.agents/skills/` — installable community skills, loaded fresh from disk by the `skill` tool
- `~/.ciel/hooks/` — Ciel hook scripts; on this runtime they are **invoked explicitly by Ciel**, not registered with the host (see Hook Support)
- `~/.ciel/skills/` — installed skills (including ciel.skill unpacked)

## Installation Footprint

At init (see `init/INIT.md`), Ciel:

1. Drops her instruction block into `AGENTS.md` at the project root (or creates it).
2. Keeps her pre-flight risk gate **inlined**: before each mutating tool call she evaluates `risk/policy.json` classification herself (allow / deny / ask / defer) — this runtime has no pre-tool hook channel. See `HOOKS` note below.
3. Injects a compact identity block into the global context (`~/.ciel/context.md`).
4. Does **not** write attribution flags: the host already emits its own trailers, and the no-attribution mandate (artifacts never mention Ciel, the Council, or the host) is enforced in-conversation by the adapter, not via runtime config.

## Platform Agnosticism Implementation

This adapter conforms to `adapters/PLATFORM_AGNOSTIC_CONTRACT.md` by:

1. **Internal Normalization:** All paths surfaced to Ciel's core are POSIX `/`; `~` is normalized to the *session* `$HOME`, which on common Freebuff hosts differs from the user's real home (e.g. Android/Termux: the CLI runs inside an Ubuntu proot-distro container with `HOME=/root` while the user's files live under `/data/data/com.termux/files/home`, bind-mounted at the same absolute path). The adapter records both and resolves user-file paths against the **user** home.
2. **Universal Command Mapping:** `shell()` maps Ciel's core commands (`ls`, `grep`, `awk`, …) through the session shell; non-POSIX hosts use the shim in `seed_skills/shell/PLATFORM_AGNOSTIC_MAPPING.md`.
3. **Shell Environment:** Commands run in discrete sessions with `cwd`/`timeout` control and optional background mode; long jobs use `BACKGROUND` and are verified via log inspection and process checks, never assumed alive. Output is normalized to stdout/stderr/exit-code triplets.
4. **Encoding:** All file reads/writes are UTF-8 strict.

## Contract Implementation

Each logical function from `adapters/ADAPTER_CONTRACT.md` maps to a Freebuff native primitive:

| Adapter contract function | Freebuff mechanism | Notes |
| --- | --- | --- |
| `load_skill(path)` | `skill` tool (by name) — reads SKILL.md fresh from disk each call | Skills live in `.agents/skills/` or `~/.ciel/skills/` |
| `spawn_subagent(name, input)` | `skill`-loaded persona + in-context deliberation (serial) | Degraded floor clause; no native dispatch tool |
| `invoke_mcp(server, tool, args)` | shell-launched stdio MCP server + JSON-RPC pipe (degraded route) | No native MCP client tool; prefer HTTP-exposed MCP servers |
| `shell(cmd, cwd, env, timeout)` | `run_terminal_command` (SYNC / BACKGROUND) | `cwd` param, timeout budgeting, POSIX bash on every host |
| `fs_read(path)` | `read_files` (multi-path, offset/limit windows) | Large files auto-truncated with line count |
| `fs_write(path, content)` | `write_file` | Full-content create/overwrite |
| `fs_edit(path, old, new)` | `str_replace` | Exact-match replacement, batched, `allow_multiple` |
| `fs_search(pattern)` | `code_search` (ripgrep) + `glob` + `list_directory` | Line-numbered matches for targeted reads |
| `context_inject(scope, content)` | `AGENTS.md` (project) / `~/.ciel/context.md` (global) | Persistent, per-project |
| `hook_register(event, handler)` | **not supported** — returns stub; gate is inlined | Ciel invokes `~/.ciel/hooks/` scripts explicitly where an event matters |
| `plan_mode(enabled)` | soft: `write_todos` plan + describe-before-execute prompt pattern | No mutation freeze guarantee |
| `escalate(prompt)` | `ask_user` (radio/checkbox/custom text) | Blocks until user responds |
| `track(tasks)` | `write_todos` | Ordered, status-tracked step list |

## Hook Support

Freebuff exposes **no lifecycle hook registration** (no pre/post tool-use interception channel). Ciel compensates:

- **Pre-flight gate (inlined):** before every mutating tool call (`write_file`, `str_replace`, `run_terminal_command`), Ciel classifies the action against `risk/policy.json` and applies allow / deny / ask / defer herself. The Mandatory Pre-Tool Council Gate for `core/`, `self_improvement/`, `configuration/global/` targets still applies — it is executed as an explicit in-conversation step, not an interception.
- **Post-execution scoring (inlined):** after mutating calls, Ciel scores the outcome and appends to `~/.ciel/activity.log` via shell.
- **Session bootstrap (inlined):** at session start Ciel re-reads her identity block from `AGENTS.md` / `~/.ciel/context.md` and re-runs `router/RUNTIME_DETECTION.md`.
- Explicit invocation: where an event matters, Ciel runs the corresponding `~/.ciel/hooks/` script directly via `shell()` — hooks stay functional as *called functions* rather than *registered callbacks*.

## Council Invocation

No parallel subagents → the Council of Five runs **sequentially, in-context**, with the degraded inline mode **explicitly declared** in `spawn_receipts.json` (`mode: "inline"`). Verification reports `unverified_member_isolation`; this is accepted on this runtime and must never be silently upgraded. See `COUNCIL_INVOCATION.md`.

## Route Map

| Ciel route | Freebuff mechanism |
| --- | --- |
| Skill activation | `skill` tool |
| Subagent (nested) | skill-loaded persona, in-context, serial |
| Parallel dispatch | not available — sequential batching |
| Pre-flight gate | inlined risk classification per mutating call |
| Post-execution scoring | inlined scoring + `activity.log` append |
| MCP | shell-launched stdio JSON-RPC (degraded) |
| Context injection | `AGENTS.md` + `~/.ciel/context.md` |
| Long task | `write_todos` plan + `BACKGROUND` shell + checkpoints to disk |
| Shell isolation | task-scoped authorization; `ask_user` for out-of-scope actions |
| Web research | `web_search` + `read_url` |
| Task tracking | `write_todos` |
| User escalation | `ask_user` (+ `suggest_followups` post-task) |
| File search | `code_search` (ripgrep) + `glob` + `list_directory` |
| Structured edit | `str_replace` / `write_file` |

## Contract Testing

This adapter ships with the capability flags above. At init, Ciel runs a self-test verifying the floor (skills, subagents, mcp, shell, fs, context): skill loading is probed with the `skill` tool, shell with a `bash` round-trip, fs with a temp-file write/read, context with an `AGENTS.md` stat, subagent delegation with a persona skill load, and MCP is probed but may fail → degraded route (in-context stub) is substituted automatically. Failures are logged; degraded-mode routes are substituted. See `adapters/ADAPTER_CONTRACT.md` § Contract Testing.
