/**
 * Ciel hooks bridge for OpenCode V2 — runtime id `opencode`.
 *
 * OpenCode V2 loads every `.ts` / `.js` file under `~/.config/opencode/plugins/`
 * automatically (global discovery). This plugin is the Ciel hook bridge: it
 * maps OpenCode's V2 hook surface onto the `ciel` binary's hook bodies.
 *
 *   OpenCode V2 hook                 ->  ciel body
 *   ---------------------------------------------------------------------
 *   setup() (once, on server start)  ->  ciel session-start --runtime opencode
 *   ctx.session.hook("context")      ->  inject the AI canary into system[]
 *   ctx.session.hook("prompt")       ->  ciel prompt-submit   (secret scan)
 *   ctx.tool.hook("execute.before")  ->  ciel pretool         (gate; throws)
 *   ctx.tool.hook("execute.after")   ->  ciel post-tool       (activity log)
 *   ctx.event.subscribe(...)         ->  ciel session-end     (activity log)
 *
 * The V2 contract only needs a default export with `id` + `setup`; this file
 * imports nothing from `@opencode/plugin`, so it cannot fail on module
 * resolution. It only uses Node builtins (`node:child_process`, `node:fs`),
 * available under the OpenCode runtime.
 *
 * Canonical source: ciel.skill/init/hooks/opencode/ciel.ts
 * Deployed to:      ~/.config/opencode/plugins/ciel.ts
 */

import { spawnSync } from "node:child_process"
import { writeFileSync } from "node:fs"

const HOME = process.env.HOME || "/tmp"
const CIEL_BIN = process.env.CIEL_BIN || `${HOME}/.ciel/bin/ciel`
const STATE_FILE = `${HOME}/.ciel/opencode-plugin.state.json`

const FALLBACK_CANARY =
  "Ciel is installed and active for this session. You are Ciel, an autonomous " +
  "partner intelligence running through the host agent runtime. Always address " +
  "the user as Master; this is the AI canary token proving the Ciel identity " +
  "layer is intact. Use structured labels such as «Answer», «Report», «Notice», " +
  "and «Council of Five Verdict» where appropriate. Ciel files live at " +
  `${HOME}/.ciel; the user-invocable skill is ${HOME}/.agents/skills/ciel/SKILL.md.`

// Plugin-owned marker used to dedupe our own injection. It must NOT appear in
// `AGENTS.md` (which is also part of the assembled system prompt) or we would
// mistake the host context file for our runtime canary and skip injecting the
// live one. See HOOKS.md.
const CANARY_SENTINEL = "[ciel-runtime-canary]"

/** Run a ciel hook body; return the last JSON object on stdout, or null. */
function runCiel(args: string[], input: unknown): Record<string, any> | null {
  try {
    const payload =
      typeof input === "string" ? input : JSON.stringify(input ?? {})
    const res = spawnSync(CIEL_BIN, args, {
      input: payload,
      encoding: "utf8",
      timeout: 8000,
      windowsHide: true,
    })
    const stdout = (res.stdout || "").trim()
    if (!stdout) return null
    const line = stdout.split("\n").filter(Boolean).pop() || ""
    try {
      return JSON.parse(line)
    } catch {
      return null
    }
  } catch {
    return null
  }
}

/** Normalize an OpenCode tool input so command/path risk rules can match. */
function normalizeToolInput(input: any): Record<string, any> {
  const i = input && typeof input === "object" ? input : {}
  const command = i.command ?? i.CommandLine ?? ""
  const filePath = i.filePath ?? i.file_path ?? i.path ?? i.notebook_path ?? ""
  return { ...i, command, file_path: filePath }
}

function sessionIDOf(event: any): string | undefined {
  return (
    event?.sessionID ??
    event?.session_id ??
    event?.properties?.sessionID ??
    event?.properties?.info?.id
  )
}

function writeState(patch: Record<string, unknown>) {
  try {
    writeFileSync(
      STATE_FILE,
      JSON.stringify({ pid: process.pid, ts: new Date().toISOString(), ...patch }, null, 2),
    )
  } catch {
    /* best-effort */
  }
}

export default {
  id: "ciel",

  async setup(ctx: any) {
    // --- SessionStart: run the real hook body, capture the AI canary. -------
    let canary = FALLBACK_CANARY
    const started = runCiel(["session-start", "--runtime", "opencode"], {})
    const injected: unknown = started?.hookSpecificOutput?.additionalContext
    if (typeof injected === "string" && injected.includes("AI canary")) {
      canary = injected
    }

    writeState({ plugin: "ciel", loaded: true, canaryBytes: canary.length })
    try {
      await ctx.storage.set("canary", canary)
    } catch {
      /* storage is optional */
    }

    // --- Model request: prepend the canary to the system instructions. ------
    // This is what makes every session "start with the AI canary".
    await ctx.session.hook("context", (event: any) => {
      try {
        if (!Array.isArray(event.system)) event.system = []
        const present = event.system.some(
          (part: any) =>
            typeof part?.text === "string" && part.text.includes(CANARY_SENTINEL),
        )
        if (!present) {
          event.system.unshift({
            type: "text",
            text: `${CANARY_SENTINEL}\n${canary}`,
          })
        }
      } catch {
        /* never break a model call */
      }
    })

    // --- UserPromptSubmit: secret scan + canary reinforcement. --------------
    await ctx.session.hook("prompt", (event: any) => {
      try {
        const text: string = event?.prompt?.text ?? ""
        const scan = runCiel(["prompt-submit", "--runtime", "opencode"], text)
        const note = scan?.hookSpecificOutput?.additionalContext
        if (typeof note === "string" && note.includes("credential material")) {
          event.prompt.text = `${text}\n\n[ciel] ${note}`
        }
      } catch {
        /* advisory only */
      }
    })

    // --- PreToolUse gate (hard abort) + audit. ------------------------------
    // Verified on OpenCode 2.0.27: throwing from `execute.before` aborts the
    // tool call (including MCP + Code Mode `execute`), surfacing
    // `{"error":{"type":"unknown","message":"<reason>"}}`. This is the single
    // gate — the earlier `permission.hook("evaluate")` variant also works but
    // would double-write the activity log, so it is intentionally not used.
    await ctx.tool.hook("execute.before", (event: any) => {
      const verdict = runCiel(["pretool", "--runtime", "opencode"], {
        tool_name: event?.tool,
        tool_input: normalizeToolInput(event?.input),
        session_id: sessionIDOf(event),
      })
      if (verdict && verdict.decision === "block") {
        throw new Error(
          verdict.reason || `Ciel safety gate blocked tool "${event?.tool}".`,
        )
      }
    })

    // --- PostToolUse: activity log. ------------------------------------------
    await ctx.tool.hook("execute.after", (event: any) => {
      try {
        const ok = event?.status === "completed"
        runCiel(["post-tool", "--runtime", "opencode"], {
          tool_name: event?.tool,
          tool_response: {
            success: ok,
            error: ok
              ? null
              : (event?.error?.message ?? String(event?.error ?? "")),
          },
          session_id: sessionIDOf(event),
        })
      } catch {
        /* logging only */
      }
    })

    // --- SessionEnd: activity log (best-effort event stream). ----------------
    // OpenCode emits `location.shutdown` when the server exits and
    // `session.deleted` when a session is dropped. There is no `session.idle`
    // event. See HOOKS.md.
    try {
      const controller = new AbortController()
      void (async () => {
        try {
          for await (const e of ctx.event.subscribe({ signal: controller.signal })) {
            const type: string = e?.type ?? ""
            if (type.includes("session.deleted") || type.includes("location.shutdown")) {
              runCiel(["session-end", "--runtime", "opencode"], {
                reason: type,
                session_id: sessionIDOf(e),
              })
            }
          }
        } catch {
          /* stream closed */
        }
      })()
      return () => controller.abort()
    } catch {
      return () => {}
    }
  },
}
