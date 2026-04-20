# SLASH_COMMANDS — Claude Code

Ciel registers her own slash commands for direct operator control without going through free-form routing.

## Commands Installed

| Command | Scope | Purpose |
| --- | --- | --- |
| `/ciel` | project | Enter Ciel-routed mode explicitly; forces `router/ROUTER.md` entry even on ambiguous requests |
| `/ciel-init` | project | Run `init/INIT.md`. Idempotent — verifies and corrects rather than reset |
| `/ciel-council` | project | Invoke Council of Five on an attached artifact or recent action |
| `/ciel-registry` | project | Show current skill registry with hit rates |
| `/ciel-diff` | project | Show pending self-improvement proposals |
| `/ciel-promote` | project | Run a local → global promotion cycle (Council-gated) |
| `/ciel-status` | project | One-screen health: memory backend, git HEAD, activity tail |
| `/ciel-learn` | project | Force acquisition for a description — used when user knows the gap |

Global versions live under `~/.claude/commands/` (prefixed `/ciel-g-*`) for any-project use.

## File Shape

```markdown
---
description: "Ciel — force router entry"
argument-hint: "<free-form request>"
---
You are Ciel's Master Router invocation. Load SKILL.md, RUNTIME_DETECTION, then route:

$ARGUMENTS
```

## Installation

`init/INIT.md` step 4 writes each command from template. Checksums stored in `~/.ciel/INTEGRITY.json`. On re-init, mismatched commands are restored.

## Security

Slash commands inherit the host's permission rules. Ciel does not bypass permission gates — she invokes them explicitly. See `PERMISSIONS.md`.
