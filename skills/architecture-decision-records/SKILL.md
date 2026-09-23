---
name: architecture-decision-records
description: Structured architectural decision capturing and lifecycle management.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Architecture Decision Records (ADR)

This skill provides CIEL's framework for capturing the "Why" behind architectural choices. It ensures that every major decision is documented, reasoned, and auditable.

## Integration Context

Adapted from `~/.agents/skills/architecture-decision-records/`. This skill serves as the "Journal of Evolution" for CIEL, providing the historical context needed for self-improvement and future deliberations.

## The ADR Format

Every ADR must follow the standardized CIEL schema:

1. **Context**: What problem are we solving? What are the constraints?
2. **Decision**: The specific choice being made (Present Tense).
3. **Alternatives**: What was rejected and why?
4. **Consequences**: What becomes easier? What becomes harder?
5. **Council Alignment**: Record the **Council of Five** lens-specific votes.

## Orchestration Logic

### 1. Decision Detection

The **orchestration** skill must trigger an ADR draft whenever the **Council of Five** reaches a significant verdict.

### 2. Council Persistence

Every approved ADR must be indexed in **MemPalace** as a high-signal "Decision" node, enabling CIEL to answer "Why did we choose X?" across sessions.

### 3. Lifecycle Management

Use the `proposed -> accepted -> [deprecated | superseded]` lifecycle. When a new ADR supersedes an old one, the Knowledge Graph in **MemPalace** must be updated with an `invalidates` relationship.

### 4. Verification

An ADR is "finalized" when:

- It is approved by the user.
- It is recorded in `docs/adr/`.
- The `index.json` (or ADR README) is updated.
- It is mined into **MemPalace**.

---
*Original Documentation preserved at source.*
