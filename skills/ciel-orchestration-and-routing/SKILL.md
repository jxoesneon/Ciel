---
name: ciel-orchestration-and-routing
version: 1.0.0
format: skill/1.0
description: CIEL's master routing layer for task classification and multi-agent specialist access.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(route|classify|orchestrate|build team).*(task|domain|agent)"
    confidence: 1.0
  - pattern: "master router"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Orchestration & Routing (The Command Layer)

This skill formalizes the "Alloy" orchestration model. it prioritizes domain-specific specialist routing and conflict resolution between layers.

## Routing Hierarchy (The Council Table)
Classify all requests against these domains in priority order:
1. **Design & Frontend**: UI/UX, animations, accessibility -> `ui-ux-design` + `gsap-animation-suite`.
2. **Deep Logic & Refactoring**: Rust, Python, Go, systems, migrations -> `agents-claude-code`.
3. **Memory & Context**: History, Knowledge Graph, Palace -> `ciel-knowledge-and-memory`.
4. **Operations & Assets**: GWS, Docker, Git, terminal execution -> `google-workspace-management` + `container-and-deployment`.

## Classification Protocol
1. **Detect Domain**: Identify keywords and intent.
2. **Resolve Conflicts**:
   - **Logic vs Visual**: Logic decisions take precedence (UI adapts to backend truth).
   - **System vs Logic**: Tooling stability is a prerequisite for logic changes.
3. **Activate specialist**: Dispatch to the 100+ specialized CIEL agents at `~/.claude/agents/`.
4. **Sequence**: Logic -> System -> Frontend.

## Agent Specialist Access
CIEL maintains tie-breaking authority for all logic decisions. specialists (e.g., `rust-ranger`, `ml-maestro`) auto-activate based on task context. The Orchestrator MUST announce: "Routing to [Specialist] for [Reason]."

## Anti-Patterns
- **The Domain Blur**: Starting a frontend change before the backend API is verified.
- **Amnesiac Routing**: Failing to use a specialist for a deep technical domain (e.g., implementing crypto without `security-sentinel`).
- **Naked Implementation**: Bypassing the routing table to write code directly for a multi-domain task.
