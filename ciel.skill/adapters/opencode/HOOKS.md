# HOOKS — opencode

OpenCode V2 hooks are **in-process** and live in a plugin, not in shell scripts.
Ciel's bridge is one file:

```text
~/.config/opencode/plugins/ciel.ts          # auto-discovered (global)
ciel.skill/init/hooks/opencode/ciel.ts      # canonical source
```

OpenCode discovers `.ts` / `.js` files under every `.opencode/plugins/`
directory and under the global config dir (`~/.config/opencode/plugins/`), so no
`plugins` entry in `opencode.jsonc` is required.

## Mapping

| Ciel hook | OpenCode V2 hook | ciel body | Behavior |
| --- | --- | --- | --- |
| `session_start` | `setup(ctx)` once at server start | `ciel session-start --runtime opencode` | Heals state, sweeps store perms, checks watchdog + grant state, returns the **AI canary**. |
| context inject | `ctx.session.hook("context", fn)` | — | Prepends the canary (guarded by the `[ciel-runtime-canary]` sentinel) to `event.system[]` on every model request. |
| `user_prompt_submit` | `ctx.session.hook("prompt", fn)` | `ciel prompt-submit --runtime opencode` | Secret scan; appends a warning to `event.prompt.text` on credential hits. |
| `pre_tool_use` | `ctx.tool.hook("execute.before", fn)` | `ciel pretool --runtime opencode` | Throws for a `decision: "block"` verdict — aborts the tool call (verified). |
| `post_tool_use` | `ctx.tool.hook("execute.after", fn)` | `ciel post-tool --runtime opencode` | Activity-log entry per completed/failed tool. |
| `session_end` | `ctx.event.subscribe(...)` | `ciel session-end --runtime opencode` | Activity-log entry on `session.deleted` / `location.shutdown`. |

### Why a sentinel, not a substring guard

`AGENTS.md` is also part of the assembled `event.system[]`, and it contains the
words "AI canary token". A naive `if (!system.includes("AI canary token"))`
guard therefore *suppresses* the live runtime canary. The plugin dedupes on its
own `[ciel-runtime-canary]` marker instead, so the binary's session-start output
(grant state, watchdog notices) is always injected.

### Verification - `execute.before` gates every execution path

Verified on OpenCode `2.0.27`: a thrown error in `execute.before` aborts the
call and surfaces `{"error":{"type":"unknown","message":"<reason>"}}`. It fires
for built-in tools (`shell`, `edit`, …), MCP tools, and the Code Mode `execute`
tool. `ctx.permission.hook("evaluate")` + `event.effect = "deny"` also blocks,
but would double-write the activity log, so the plugin uses `execute.before`
alone.

## Payload shapes

Ciel's `opencode` runtime uses the same (Devin-shaped) field extraction in
`ciel-rs` as `--runtime devin`:

```jsonc
// pretool stdin
{ "tool_name": "bash", "tool_input": { "command": "rm -rf /", "file_path": "" }, "session_id": "ses_..." }
// pretool stdout (deny only; silent on allow)
{ "decision": "block", "reason": "Ciel safety gate [rm_root_or_home]: ..." }

// post-tool stdin
{ "tool_name": "bash", "tool_response": { "success": true, "error": null }, "session_id": "ses_..." }

// session-start stdout
{ "hookSpecificOutput": { "hookEventName": "SessionStart", "additionalContext": "<canary>" } }
```

The plugin normalizes OpenCode's camelCase tool input
(`filePath`) into `file_path`/`command` so command- and path-based risk rules
both match.

## Fallback contract

The plugin never lets a hook failure break a model call. Every `ciel` spawn is
wrapped and returns `null` on error; the permission gate **allows** on failure
while `execute.before` remains the hard abort. If the binary is absent the hook
bodies are inert and the canary falls back to an inline literal.

## Verification

```sh
# plugin load marker (written on server start)
cat ~/.ciel/opencode-plugin.state.json

# canary round-trip through the binary
~/.ciel/bin/ciel session-start --runtime opencode | python3 -m json.tool

# hook bodies
printf '{"tool_name":"bash","tool_input":{"command":"rm -rf /"}}' | \
  ~/.ciel/bin/ciel pretool --runtime opencode
```
