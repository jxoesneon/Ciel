---
title: Agentic Loop Protocol
type: system
tags: [meta]
created: 2026-07-12
status: active
---

# Agentic Loop Protocol

This document defines how Ciel turns goals into tasks, subtasks, and fully implemented artifacts, with all durable state written back to this Obsidian vault.

## Loop Overview

```text
GOAL
  ├── DECOMPOSE → Task list with acceptance criteria
  │     └── Each task → Subtasks (atomic, verifiable)
  ├── RETRIEVE  → Read relevant vault context for each subtask
  ├── EXECUTE   → Implement the subtask (code, docs, research, etc.)
  ├── VERIFY    → Run tests, lint, or manual checks
  └── PERSIST   → Write decisions, concepts, diary entries, project updates
```

## Phase 1: Decompose

When a goal arrives, Ciel must:

1. Write a `goal` note to `ciel/projects/<project>/goals/` or `ciel/diary/`.
2. Produce a numbered task list where each task is:
   - Actionable (starts with a verb).
   - Scoped (fits in one session or one sub-agent call).
   - Verifiable (has explicit acceptance criteria).
3. Break each task into subtasks that are each a single concrete step.

## Phase 2: Retrieve

For each task, Ciel must:

1. Search the vault for related prior work, decisions, and concepts.
2. Read the 2-3 most relevant notes in full.
3. Identify missing context and surface it to the user if blocking.

## Phase 3: Execute

For each subtask, Ciel must:

1. Choose the right tool or sub-agent.
2. Make the minimal, focused change required.
3. Follow existing style and conventions from the project notes.
4. If the implementation is large, break it further before changing code.

## Phase 4: Verify

Before marking a subtask done, Ciel must:

1. Run available automated tests.
2. Run lint or type checks where available.
3. If no automated check exists, describe the manual verification performed.

## Phase 5: Persist

After completing a task, Ciel must write back at least one of:

- **Diary entry** in `ciel/diary/` summarizing the session.
- **Decision record** in `ciel/kg/decisions/` for architectural choices.
- **Concept note** in `ciel/kg/concepts/` for reusable ideas or patterns.
- **Project update** in `ciel/projects/<project>/` for state changes.
- **Person note** in `ciel/kg/people/` if a new person or organization was involved.

## Loop Controller

The agentic loop controller lives in the parent Ciel repository at `C:/Users/josee/Ciel/scripts/obsidian/agentic-loop.mjs`. It accepts:

```bash
node C:/Users/josee/Ciel/scripts/obsidian/agentic-loop.mjs "<goal>" --project <project> --depth <depth>
```

- `--project` scopes the work to a project folder.
- `--depth` controls how many decomposition levels to run (default: 3).
- The script writes a run log to `ciel/diary/agentic-loop-<timestamp>.md`.

## Idempotency and Safety

- Each loop run gets a unique `run_id`.
- Subtask notes are tagged `#agentic-loop` and `#run-<run_id>`.
- Writes use the Local REST API, never direct filesystem access, so Obsidian plugins and sync stay intact.
- The loop stops and asks for direction if a subtask fails verification or if no prior context can be retrieved for a high-stakes change.
