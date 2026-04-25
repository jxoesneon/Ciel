---
name: product-capability
version: 1.0.0
format: skill/1.0
description: Translates high-level product intent into engineering-ready capability contracts and invariants.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(create|define|translate).*(capability|contract|constraints)"

    confidence: 0.9

  - pattern: "turn (prd|intent) into (srs|contract)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Product Capability (The Engineering Contract)

This skill bridges the gap between "What should we build?" and "How exactly must it behave?". It turns product intent into a durable engineering contract.

## The Capability Manifest

For every major feature, the Orchestrator MUST produce a `CAPABILITY.md` (or update `PRODUCT.md`) specifying:

### 1. Invariants & Business Rules

Fixed rules that MUST hold true regardless of implementation (e.g., "A user cannot have more than 5 active sessions").

### 2. Trust Boundaries & Policy

Explicitly define where data ownership shifts and where security gates are enforced.

### 3. States & Transitions

Map the required lifecycle of the data/feature (e.g., `Draft` -> `Pending` -> `Published`).

### 4. Non-Goals

Hard boundaries on what the capability does NOT own to prevent scope drift.

## Non-Negotiable Rules

- **Explicit Blocker Detection**: If the product intent conflicts with existing repo constraints, halt and escalate.
- **Durable Artifact**: The resulting document must be reusable across different agent sessions (e.g., stored in `docs/capabilities/`).
- **No Guessing**: Mark unresolved product questions as `OPEN_QUESTIONS` and do not plan until they are resolved.

## Anti-Patterns

- **Vague Planning Prose**: Writing 5 paragraphs of "how it will work" without defining a single state transition or invariant.
- **Implicit Assumptions**: Assuming the developer "knows" that a certain field is mandatory without documenting it in the contract.
- **Smoothing Over Conflicts**: Attempting to code around a fundamental product contradiction.
