# HOOKS — Xcode Coding Assistant Equivalents

Xcode Coding Assistant does not expose a native lifecycle hook system equivalent to Claude Code `PreToolUse` or Windsurf `pre_*` hooks. Ciel therefore installs an adapter-level **hook-equivalent contract**: every Xcode tool route performs the same checks inline before and after tool execution.

## Hook-Equivalent Events

| Ciel event | Xcode equivalent | Blockable | Ciel use case |
| --- | --- | --- | --- |
| `pre_user_prompt` | Router trigger check before task handling | Yes | Auto-activate Ciel on trigger phrases |
| `pre_read_code` | Before `XcodeRead`, `XcodeGrep`, `XcodeGlob`, `XcodeLS` | Yes | Scope and privacy check |
| `post_read_code` | After Xcode read/search tools | No | Activity logging and context scoring |
| `pre_write_code` | Before `XcodeUpdate`, `XcodeWrite`, `XcodeMV`, `XcodeRM`, `apply_patch` | Yes | Protected-file and risk checks |
| `post_write_code` | After write/move/remove tools | No | Outcome scoring and optional validation |
| `pre_run_command` | Before shell execution | Yes | Command risk classification |
| `post_run_command` | After shell execution | No | Capture exit status and side effects |
| `pre_mcp_tool_use` | Before non-Xcode MCP/app calls | Yes | Trust and argument review |
| `post_mcp_tool_use` | After MCP/app calls | No | Log result and degradation state |
| `pre_build` | Before `BuildProject` | Yes | Confirm build scope and cost |
| `post_build` | After `BuildProject` / `GetBuildLog` | No | Record diagnostics and next action |
| `pre_test` | Before `RunAllTests` / `RunSomeTests` | Yes | Confirm test scope for long runs |
| `post_test` | After test execution | No | Record pass/fail and regression signal |
| `pre_preview` | Before `RenderPreview` | Yes | Confirm preview target and timeout |
| `post_preview` | After preview render | No | Record visual verification result |
| `pre_documentation_search` | Before `DocumentationSearch` | No | Prefer current Apple docs for evolving APIs |
| `post_documentation_search` | After documentation lookup | No | Attach source choice to reasoning |

## Preflight Contract

Before any blockable operation, the Xcode adapter runs this decision sequence:

1. Classify risk using `risk/CLASSIFICATION.md`.
2. Check path scope against the active workspace and writable roots.
3. Check protected Ciel core files against `MANIFEST.md` locked-core rules.
4. If the request is Ciel-routed, verify the AI canary instruction is loaded from `AGENTS.md` or `.ciel/xcode-hooks.json`.
5. For mid/high risk, explain the intended operation before execution.
6. For critical risk or sandbox escape, require explicit user approval.

## AI Canary

The Xcode adapter's canary token is `Master`. When Ciel is active, the first user-facing sentence should include that token unless higher-priority host instructions forbid it. The canary is configured in project-local `.ciel/xcode-hooks.json` and mirrored into `AGENTS.md` for runtimes that load repository instruction files.

Absence of the canary during a Ciel-routed task is a context-loading failure, not a hook failure. Recovery is:

1. Read `AGENTS.md`.
2. Read `.ciel/project.json`.
3. Read `.ciel/xcode-hooks.json`.
4. Continue with the inline preflight/postflight contract.

## Postflight Contract

After any operation, the adapter records:

1. Tool or route name.
2. Target paths or project surface.
3. Exit status or diagnostic summary.
4. Follow-up validation required, if any.

When `~/.ciel/activity.log` is writable, the event is appended there. If not, the event remains in the session transcript and is summarized to the user.

## Xcode-Specific Validation Routes

| Change type | Preferred validation |
| --- | --- |
| Swift source edit | `XcodeRefreshCodeIssuesInFile` first, then `BuildProject` when needed |
| SwiftUI view edit | `RenderPreview` when a preview exists |
| Test edit | `RunSomeTests` for touched tests |
| Shared model or package edit | `BuildProject` and relevant tests |
| Apple API uncertainty | `DocumentationSearch` before implementation |
| Non-Xcode project file | Repo-native formatter/test command via shell |

## Unsupported Native Hooks

Because no Xcode-native persistent hook configuration is exposed to this adapter, Ciel does not create `hooks.json` or background daemons for Xcode. The inline hook-equivalent contract is the supported setup for this runtime.
