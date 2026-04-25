---
name: safety-guard
version: 1.0.0
format: skill/1.0
description: Primary operational failsafe. Intercepts destructive commands and enforces directory-level write freezing during autonomous orchestration.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(run|execute).*(danger|production|autonomous)"
    confidence: 0.9
  - pattern: "prevent (destructive|dangerous) (operations|commands)"
    confidence: 0.9
  - pattern: "(freeze|lock).*(directory|folder|writes)"
    confidence: 0.85

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Safety-Guard (Operational Failsafe)

This skill serves as the primary operational failsafe for CIEL's autonomous agents. It enforces the "Do No Harm" mandate by establishing strict operational boundaries, intercepting destructive actions, and allowing for granular, directory-level write locking.

## Integration Context

Adapted from `~/.agents/skills/safety-guard/`. This skill provides the behavioral ruleset that the **Auditor** sub-agent uses to evaluate `run_shell_command` requests and file modifications before execution.

## Core Protection Modes

### 1. Careful Mode (Destructive Command Interception)

The Auditor MUST intercept and **FAIL** the execution of the following shell command patterns unless explicit human confirmation is provided via `ask_user`:

- Recursive/forced deletions (e.g., `rm -rf /`, `rm -rf ~`, `Remove-Item -Recurse -Force` at root levels).
- Destructive Git operations (e.g., `git push --force`, `git reset --hard`, `git clean -fd`).
- Database annihilation (e.g., `DROP TABLE`, `DROP DATABASE`).
- Broad permission changes (e.g., `chmod 777 -R`).
- Escalated deletions (e.g., `sudo rm`).
- Publishing artifacts (e.g., `npm publish`).

**Action**: Present the command's exact intent to the user, warn of the consequences, and suggest a safer alternative (e.g., `git reset --soft` or targeting a specific file instead of `*`).

### 2. Freeze Mode (Scope Confinement)

During complex, multi-agent refactoring or targeted feature implementations, orchestration can enforce a "Freeze Mode" limiting write access to a specific directory tree.

- **Rule**: If a Freeze Mode directory is declared (e.g., `src/components/`), the Auditor MUST reject any `write_file` or `replace` actions attempting to modify files outside that directory.
- **Exception**: Read operations (`read_file`, `grep_search`, `glob`) are always universally permitted to maintain contextual awareness.

### 3. Guard Mode (Maximum Security)

Activates both Careful Mode and Freeze Mode simultaneously. Recommended for fully autonomous (unsupervised) loops.

## Orchestration Rules

- **Pre-Execution Check**: Every `run_shell_command` involving shell built-ins or system state modification must be cross-referenced against Careful Mode patterns.
- **Audit Trail**: Every blocked action MUST be logged locally (e.g., via `mempalace_diary_write` or a local `.ciel/safety-log.md`) with the agent's intent, the blocked command, and the timestamp.
