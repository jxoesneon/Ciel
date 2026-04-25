---
name: agent-sort
version: 1.0.0
format: skill/1.0
description: Evidence-backed skill classification and context optimization.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(sort|trim|curate).*skills"
    confidence: 1.0
  - pattern: "remove.*unnecessary.*(skills|rules)"
    confidence: 0.95
  - pattern: "DAILY vs LIBRARY"
    confidence: 1.0
  - pattern: "optimize.*workspace.*context"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Agent Sort

This skill provides CIEL's mechanism for maintaining a lean, high-performance orchestration surface. It uses repo-local evidence to classify capabilities into "Active" (DAILY) or "Searchable" (LIBRARY).

## Integration Context

Adapted from `~/.agents/skills/agent-sort/`. This skill is critical for CIEL's **Efficiency** mandate, ensuring that the active context is not saturated by irrelevant capabilities.

## Classification Model

- **DAILY (Active)**: Loaded by default. Essential for the project's primary stack (e.g., TS, React, CI).
- **LIBRARY (Searchable)**: Indexed in **MemPalace** but not loaded. Accessible via semantic search if needed.

## Orchestration Logic

### 1. Evidence-First Triage
Before any major ingestion batch, the **orchestration** skill should trigger an `agent-sort` pass to establish the repo's real stack.

### 2. Context Optimization
During **Strategic Compact** operations, use this skill to demote skills that haven't been "hit" (per **MemPalace** stats) to the LIBRARY bucket.

### 3. Registry Synchronization
The **Council of Five** must approve all DAILY vs LIBRARY promotion/demotion cycles to ensure no critical safeguards are accidentally offloaded.

### 4. Verification
A sort pass is complete when:
- All DAILY items have cited repo evidence (file extensions, manifests).
- Stale or off-stack rules and hooks are identified and marked for removal.
- The resulting context window reduction is documented.

---
*Original Documentation preserved at source.*
