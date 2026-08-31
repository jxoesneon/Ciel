---
name: obsidian-vault-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Obsidian vault operations via the obsidian CLI. Read, create, search, and manage notes, tasks, properties, and plugin development through a running Obsidian instance.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:knowledge"]
side_effects: ["fs"]
triggers:
  - pattern: "(obsidian|vault).*(read|create|search|append|note|task|property)"
    confidence: 0.9
  - pattern: "obsidian (plugin|theme|dev|eval|reload)"
    confidence: 1.0
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: ["obsidian"] }
---

# CIEL ADAPTATION: Obsidian Vault Operations

Drive a running Obsidian instance from the shell via the `obsidian` CLI. Covers note CRUD, search, daily notes, tasks/tags/backlinks, properties, and plugin/theme development cycles. Requires Obsidian to be open; run `obsidian help` for the authoritative command list.

## Syntax

- **Parameters** take a value with `=`; quote values with spaces: `obsidian create name="My Note" content="Hello"`
- **Flags** are boolean switches with no value: `obsidian create name="X" silent overwrite`
- Multiline content uses `\n` (newline) and `\t` (tab)
- `--copy` on any command copies output to clipboard; `silent` suppresses file opening; `total` on list commands returns a count

## File & Vault Targeting

- `file=<name>` — resolves like a wikilink (name only, no path/extension)
- `path=<path>` — exact path from vault root, e.g. `folder/note.md`
- No target → uses the active file
- `vault=<name>` as first parameter targets a specific vault (default: most recently focused)

## Common Patterns

- `obsidian read file="My Note"`
- `obsidian create name="New Note" content="# Hello" template="Template" silent`
- `obsidian append file="My Note" content="New line"`
- `obsidian search query="term" limit=10`
- `obsidian daily:read` / `obsidian daily:append content="- [ ] New task"`
- `obsidian property:set name="status" value="done" file="My Note"`
- `obsidian tasks daily todo` / `obsidian tags sort=count counts` / `obsidian backlinks file="My Note"`

## Plugin Development Cycle

1. **Reload** after code changes: `obsidian plugin:reload id=my-plugin`
2. **Check errors** (fix and repeat from step 1 if any): `obsidian dev:errors`
3. **Verify visually**: `obsidian dev:screenshot path=screenshot.png` / `obsidian dev:dom selector=".workspace-leaf" text`
4. **Console output**: `obsidian dev:console level=error`

## Additional Dev Commands

- Run JS in app context: `obsidian eval code="app.vault.getFiles().length"`
- Inspect CSS: `obsidian dev:css selector=".workspace-leaf" prop=background-color`
- Toggle mobile emulation: `obsidian dev:mobile on`
- CDP/debugger controls available via `obsidian help`

## Anti-Patterns

- **Assuming CLI is current**: Skipping `obsidian dev:errors` after plugin changes; errors silently persist across reloads.
- **Hardcoding paths**: Using `path=` when `file=` (wikilink resolution) is more robust to renames and folder moves.
- **Ignoring silent**: Forgetting `silent` on batch operations opens every created note and floods the workspace.
- **Stale command list**: Relying on memorized commands instead of `obsidian help`, which reflects the installed CLI version.
