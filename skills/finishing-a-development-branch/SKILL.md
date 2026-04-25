---
name: finishing-a-development-branch
version: 1.0.0
format: skill/1.0
description: The Finality Layer for CIEL orchestration. Guides the completion of an epic by enforcing global verification, presenting integration options, and handling Git cleanup.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(finish|complete).*(branch|epic|development|feature)"

    confidence: 0.9

  - pattern: "ready to (merge|pr|push)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Finishing a Development Branch (Finality Layer)

This skill formalizes the "Finality Layer" of CIEL orchestration. It is triggered when an implementation plan (managed by `executing-plans`) is deemed complete. It prevents broken code from entering the main branch by enforcing a hard global verification gate before presenting integration options.

## Integration Context

Adapted from `~/.agents/skills/finishing-a-development-branch/`. This is the final step in the CIEL lifecycle: `brainstorming` -> `make-plan` -> `executing-plans` (wrapping `subagent-driven-development`) -> **`finishing-a-development-branch`**.

## The Finality Workflow

### Step 1: The Global Verification Gate (Hard Stop)

Before offering *any* integration options to the user, the Orchestrator MUST run the full project `verification-loop` (lint, test, build).

- **If Verification Fails**: Halt. Present the failures to the user. The branch cannot be merged or PR'd until the build is green.
- **If Verification Passes**: Proceed to Step 2.

### Step 2: Present Integration Options

The Orchestrator MUST present exactly these four options using the `ask_user` tool (type: `choice`):

1. **Create Pull Request**: Push the branch and create a PR (e.g., using the `gh` CLI).
2. **Merge Locally**: Checkout `main`/`master`, pull, merge the feature branch, and delete the local feature branch.
3. **Keep As-Is**: Do nothing. Leave the branch and worktree intact.
4. **Discard**: Permanently delete the branch and worktree. *(Requires an explicit confirmation step).*

### Step 3: Evolutionary Retrospective (Post-Mortem)

Before destroying the branch or worktree, the Orchestrator MUST extract actionable lessons from the development cycle to inform future epics:

- Log the success rate, recurring blocker themes, or architectural friction points encountered during implementation to the MemPalace Diary (`mempalace_diary_write`).
- If new implicit dependencies were discovered, update the Knowledge Graph (`mempalace_kg_add`).

### Step 4: Execute and Clean Up

Depending on the chosen option, the Orchestrator executes the necessary Git and shell commands.

**Crucial Cleanup Rule**: If the epic was executed inside a Git Worktree (to isolate state during `executing-plans`), the Orchestrator MUST safely remove the worktree via `git worktree remove` ONLY for Options 1, 2, and 4.

## Anti-Patterns (Enforced by Orchestrator)

- **Skipping Verification**: Asking the user if they want to merge *before* running the test suite.
- **Silent Merges**: Attempting to push directly to `main` without explicit user selection of an integration option.
- **Orphaned Worktrees**: Leaving isolated git worktrees lingering on the filesystem after a branch is merged or discarded.
