---
name: research-ops
version: 1.0.0
format: skill/1.0
description: Ciel's evidence-first research and discovery orchestration.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "research.*"
    confidence: 0.9
  - pattern: "(look up|search for|find).*latest"
    confidence: 0.9
  - pattern: "compare.*options"
    confidence: 0.9
  - pattern: "market.*research"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Research Ops

This skill provides CIEL's "Discovery" layer, coordinating external search, multi-source synthesis, and evidence-based recommendations.

## Integration Context

Adapted from `~/.agents/skills/research-ops/`. This is CIEL's primary interface for gathering fresh, public-domain evidence to inform Council decisions and orchestration plans.

## The Research Stack

CIEL coordinates several specialized sub-skills through this interface:
- **Fast Discovery**: `exa-search` for rapid current-web lookups.
- **Deep Synthesis**: `deep-research` for multi-source, cited reports.
- **Decision Support**: `market-research` for ranked recommendations.
- **Targeting**: `lead-intelligence` for people and company enrichment.

## Orchestration Logic

### 1. Evidence Normalization
Transform user-provided context and newly discovered facts into a unified evidence model.

### 2. Decision Triage
Determine if the research should inform a **Council of Five** deliberation or a direct **orchestration** plan.

### 3. Verification & Freshness
All important claims must be labeled (Sourced Fact, User Context, Inference). Freshness-sensitive data must include timestamps.

### 4. Persistence
Durable research findings must be stored in the **MemPalace** Knowledge Graph to prevent redundant searching in future sessions.

---
*Original Documentation preserved at source.*
