---
title: Ciel Obsidian Brain
type: system
tags: [meta]
created: 2026-07-12
status: active
---

# Ciel Obsidian Brain

This is the local-first, markdown-native persistent memory system for Ciel. It replaces the previous memory backend with an Obsidian vault that any AI agent can read, write, and reason over.

## What It Is

- A **plain markdown vault** that lives on your device.
- A **source of truth** for Ciel's identity, decisions, project context, and learned patterns.
- A **processing substrate** for agentic loops: goals → tasks → subtasks → implementation notes.
- A **drop-in replacement** for the previous memory layer via the `obsidian-memory` backend adapter.

## How It Works

1. **Obsidian desktop app** hosts the vault and runs the `Local REST API` plugin.
2. **`obsidian-hybrid-search`** indexes the vault for semantic + full-text retrieval.
3. **`obra/knowledge-graph`** (or `obsidian-mcp-ultra`) provides graph traversal.
4. **Ciel's `obsidian-memory` backend** maps the abstract `CielMemoryBackend` API to these services.
5. **This vault** stores the actual markdown files.

## Directory Structure

```text
obsidian-brain/
├── _CLAUDE.md                  # Ciel's operating manual
├── index.md                    # Catalog and entry point
├── README.md                   # This file
├── AGENTS.md                   # Agentic loop rules
├── ciel/
│   ├── identity.md             # Core identity and preferences
│   ├── diary/                  # Session-by-session audit trail
│   ├── kg/                     # Knowledge graph atoms
│   │   ├── concepts/           # Ideas and frameworks
│   │   ├── decisions/          # Architecture decision records
│   │   └── people/             # People and organizations
│   └── projects/               # Per-project memory
├── raw/                        # Unprocessed source material
├── wiki/                       # Synthesized knowledge pages
└── templates/                  # Reusable note templates
```

## Setup

1. Install [Obsidian](https://obsidian.md/).
2. Open this folder as a vault.
3. Install community plugins:
   - `Local REST API` by coddingtonbear
   - `Templater`
   - `Dataview`
4. Generate an API key in Settings → Local REST API.
5. Configure Ciel to use the `obsidian-memory` backend.

## Agentic Loop

Run the agentic loop controller from the parent Ciel repository:

```powershell
node C:/Users/josee/Ciel/scripts/obsidian/agentic-loop.mjs "<goal>" --project <project> --depth 3
```

This script:

1. Decomposes the goal into tasks and subtasks.
2. For each subtask, retrieves relevant vault context, implements, and writes back.
3. Persists decisions, concepts, and project updates as notes.

See [[AGENTS.md]] for the full loop protocol.
