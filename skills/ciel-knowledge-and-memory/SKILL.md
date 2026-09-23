---
name: ciel-knowledge-and-memory
description: CIEL's framework for multi-layer knowledge ingestion, semantic memory search, and structural code exploration.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Knowledge & Memory (The Intel Layer)

This skill manages the flow of information from live codebase structure to durable cross-session memory.

## Knowledge Ingestion (Layers)

1. **Active (L1)**: GitHub/Linear for live roadmap and implementation truth.
2. **Short-term (L2)**: Project memory files (`~/.claude/projects/*/memory/`) for user preferences.
3. **Semantic (L3)**: MCP Memory Server for structured knowledge graphs and observations.
4. **Durable (L4)**: Dedicated knowledge base repo for curated research and synthesized docs.

## Memory Search (Index -> Timeline -> Fetch)

- **Mandate**: NEVER fetch full details without filtering first.
- **Protocol**:
  1. **Search**: Get a table of IDs and Titles to minimize tokens.
  2. **Timeline**: Interleave sessions and observations around an anchor ID to find context.
  3. **Fetch**: Batch-get only the 2-3 most relevant full observations.

## Structural Exploration (Smart Explore)

- **Map First**: Use `smart_search` or `smart_outline` (AST-based) before performing a full `read_file`.
- **Unfold**: Review signatures and unfold ONLY the implementation of specific functions/methods needed.
- **Token Efficiency**: Target 4-8x token savings vs raw file reads.

## Anti-Patterns

- **The Context Dump**: Reading 10+ full files when a structural map would suffice.
- **Amnesiac Session**: Starting work without checking MCP memory for previous decisions.
- **Durable-Code Drift**: Storing live application code in a knowledge base instead of the Git repo.
