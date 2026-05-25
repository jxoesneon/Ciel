# PERMISSIONS — Devin for Terminal

Devin for Terminal uses a **scope-based** permission system that differs from Claude Code's tool-based approach. Ciel must understand both to operate effectively.

## Scope-Based Permissions

```json
{
  "permissions": {
    "allow": [
      "Read(src/**)",           // File read glob
      "Write(src/**)",          // File write glob
      "Exec(git)",              // Command prefix match
      "Exec(npm run)",          // Multi-word prefix
      "Fetch(https://api.github.com/*)"  // URL pattern
    ],
    "deny": [
      "Exec(rm)",
      "Exec(sudo)",
      "Write(.env*)",
      "Write(*.lock)"
    ],
    "ask": [
      "Write(**/.env*)",
      "Exec(curl *)"
    ]
  }
}
```

## Evaluation Order

1. **Deny** — checked first; blocks immediately if matched
2. **Ask** — checked second; always prompts if matched (overrides allow)
3. **Allow** — checked third; auto-approves if matched
4. **Default** — no rule matched → prompt user

## Tool-Based Permissions

Devin also supports tool-level permissions:

```json
{
  "permissions": {
    "allow": ["read", "grep", "glob"],
    "deny": ["edit", "write"],
    "ask": ["exec"]
  }
}
```

Available tool names: `read`, `edit`, `grep`, `glob`, `exec`

## MCP Tool Permissions

```json
{
  "permissions": {
    "allow": ["mcp__github__list_issues"],
    "deny": ["mcp__github__delete_repo"],
    "ask": ["mcp__linear__*"]
  }
}
```

## Permission Modes

Devin has 4 runtime permission modes (set via `/mode` or `--permission-mode`):

| Mode | Read | Fetch | Exec | Edit/Write |
|------|------|-------|------|-----------|
| **Normal** (default) | Auto-approve | Ask | Ask | Ask |
| **Accept Edits** | Auto-approve | Ask | Ask | Auto-approve (in workspace) |
| **Bypass** | Auto-approve | Auto-approve | Auto-approve | Auto-approve |
| **Autonomous** (sandbox) | Auto-approve | Auto-approve* | Auto-approve* | Ask |

*Auto-approved because sandbox enforces filesystem/network limits.

## Ciel's Permission Integration

Ciel maps her risk ladder to Devin permissions:

| Ciel Risk | Devin Action |
|-----------|-------------|
| Critical (destructive) | `deny` rule + PreToolUse block hook |
| High (system writes) | `ask` rule + PreToolUse ask hook |
| Mid (network, outside project) | `ask` rule |
| Low (git, build, test) | `allow` rule + PreToolUse approve hook |

## Recommended Project Config

For a typical Ciel-guarded project:

```json
{
  "permissions": {
    "allow": [
      "Read(**)",
      "Exec(git)",
      "Exec(cargo)",
      "Exec(npm)",
      "Exec(node)",
      "Exec(npx)",
      "Exec(python3)",
      "Exec(ls)",
      "Exec(cat)",
      "Exec(find)",
      "Exec(grep)",
      "Exec(rg)"
    ],
    "deny": [
      "Exec(rm)",
      "Exec(sudo)",
      "Exec(curl *)",
      "Exec(wget *)",
      "Write(/etc/**)",
      "Write(/usr/**)",
      "Write(/bin/**)",
      "Write(.env*)",
      "Write(*.lock)"
    ],
    "ask": [
      "Write(**)",
      "Exec(docker *)",
      "Exec(kubectl *)"
    ]
  }
}
```

## Configuration Levels

| Priority | Source |
|----------|--------|
| 1 (highest) | Organization / Team settings |
| 2 | Session grants (interactive approvals) |
| 3 | `.devin/config.local.json` |
| 4 | `.devin/config.json` |
| 5 (lowest) | `~/.config/devin/config.json` |

## Sandbox

Use `devin --sandbox --permission-mode autonomous` for OS-level isolation:

```json
{
  "sandbox": {
    "allowed_domains": ["api.github.com", "registry.npmjs.org"],
    "denied_domains": [],
    "network_mode": "full"
  }
}
```

In sandbox mode:
- Shell commands auto-approve (sandbox enforces limits)
- Direct `edit`/`write` tool calls still prompt
- Granting a `Write(...)` scope dynamically expands the sandbox
