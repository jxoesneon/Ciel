---
name: automation-audit-ops
version: 1.0.0
format: skill/1.0
description: An evidence-first audit protocol for inventorying, classifying, and rationalizing workspace automations.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(audit|inventory|list).*(automation|job|workflow|hook|connector)"
    confidence: 0.9
  - pattern: "what (automations|hooks) are (live|broken)"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Automation Audit Ops (Inventory & Rationalization)

This skill dictates how the Orchestrator inventories and rationalizes workspace automations (CI jobs, hooks, connectors, MCP servers). It focuses on evidence-backed classification to drive "Keep / Merge / Cut / Fix-Next" decisions.

## The Audit Workflow

### 1. Evidence-First Inventory
The Orchestrator MUST NOT assume state from configuration files. It MUST verify truth by:
- Inspecting active Git hooks (`.git/hooks`).
- Checking live GitHub Action run histories.
- Probing MCP server connectivity.
- Verifying authentication tokens for connectors.

### 2. State Classification
Every identified automation MUST be classified into one of these states:
- **Active**: Configured, authenticated, and recently verified as passing.
- **Degraded**: Configured but failing (broken code or transient error).
- **Stale**: Authenticated but no longer producing useful output.
- **Broken**: Authentication expired or configuration invalid.
- **Redundant**: Overlaps significantly with another active automation.

### 3. The Rationalization Matrix
For every degraded, stale, or redundant automation, the Orchestrator MUST produce a recommendation:
- **Keep**: The automation is vital and functional.
- **Merge**: Consolidate overlapping surfaces into a single canonical lane.
- **Cut**: Delete stale or entirely redundant automations.
- **Fix-Next**: Prioritize immediate repair of a high-signal broken path.

## Integration Context
- **Workspace Audit**: Uses `workspace-surface-audit` as the primary discovery tool.
- **Verification**: Uses the `verification-loop` to confirm the state of "Fix-Next" items.
- **Knowledge**: Records the final inventory in the MemPalace Knowledge Graph for future reference.

## Anti-Patterns
- **Theorizing**: Claiming a tool is working based on its documentation without checking live logs.
- **Fix-First**: Attempting to repair automations before completing the global inventory and rationalization.
- **Silent Redundancy**: Allowing multiple overlapping automations to run, creating noise and conflicting state changes.
