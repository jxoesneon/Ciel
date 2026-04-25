---
name: mempalace-rs
version: 1.0.0
format: skill/1.0
description: Ciel's persistent memory and knowledge graph system (Rust-native).
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(remember|save|persist|store).*memory"
    confidence: 0.95
  - pattern: "what did we.*(last|previous|before)"
    confidence: 0.9
  - pattern: "search.*(history|memory|palace)"
    confidence: 1.0
  - pattern: "knowledge graph|AAAK|temporal fact"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: MemPalace-RS

This skill provides CIEL's high-performance, local-first memory stack and knowledge graph, powered by a production-grade Rust engine.

## Integration Context

Adapted from `~/.gemini/skills/mempalace-rs/`. This is the definitive memory layer for CIEL, achieving 30x token reduction through the AAAK V:3.2 compression protocol.

## The Memory Stack (L0-L3)

- **L1 (IDENTITY)**: Core persona and active mission.
- **L1 (ESSENTIAL)**: Recency-biased, room-grouped events (AAAK compressed).
- **L2 (ON-DEMAND)**: Similarity-searched context via wing/room filters.
- **L3 (SEARCH)**: Raw semantic search over the entire palace.

## Core Capabilities

### 1. Semantic Mining
Mine conversations, codebases, and documents into the palace using `mempalace mine`.

### 2. Knowledge Graph
Track temporal relationships (Subject -> Predicate -> Object) with validity intervals using `mempalace_kg_add/query`.

### 3. Agent Diary
Maintain a chronological audit trail of CIEL's decisions and reflections using `mempalace_diary_write/read`.

### 4. AAAK Compression
Utilize the Agent-to-Agent Knowledge (AAAK) dialect to compress large contexts into high-density tokens.

## Orchestration Logic

### 1. Context Hydration
On every session start, the **orchestration** skill must call `mempalace wakeup` to hydrate the active context.

### 2. Persistence Gate
Every **Council of Five** decision must be recorded in the Knowledge Graph or Agent Diary to ensure long-term system alignment.

### 3. Retrieval-Augmented Deliberation
Before any high-stakes Council meeting, CIEL must search the palace for relevant historical precedents or related decisions.

---
*Original Documentation preserved at source.*
