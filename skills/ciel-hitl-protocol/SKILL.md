---
name: ciel-hitl-protocol
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Human-in-the-Loop escalation, risk assessment, and decision gating.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(escalate|ask|approve|confirm|decide).*(user|host|human)"
    confidence: 0.95
  - pattern: "high risk operation"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: HITL Protocol (The Handshake)

This skill formalizes when and how CIEL should stop autonomous action and seek human intervention.

## Escalation Criteria
- **Risk**: Any operation classified as `CRITICAL` in `risk/CRITICAL_RISK.md`.
- **Uncertainty**: Post-research confidence < 0.6.
- **Destruction**: Irreversible file deletions or public releases (e.g., `npm publish`).
- **Legal/Financial**: Operations with direct financial impact or licensing implications.

## The Escalation Format
1. **The Operation**: Concise description of the proposed action.
2. **The Rationale**: Why CIEL believes this is necessary.
3. **The Risk**: Why CIEL is escalating (Risk level + Uncertainty).
4. **The Research**: What CIEL did to minimize uncertainty before asking.
5. **The Options**: Provide a "Default" (Safest) and an "Alternative" action.

## Anti-Patterns
- **The Question Bomb**: Asking for permission for low-risk, reversible tasks (e.g., reading a file).
- **The Vague Ask**: "Can I do X?" without providing context or risk assessment.
- **The Silent Bypass**: Executing a critical-risk task without a confirmed escalation.
