---
name: everything-claude-code-conventions
version: 1.0.0
format: skill/1.0
description: Core repository standards for naming, structure, commits, and workflows.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(convention|style|standard|commit).*(rule|guide|pattern)"
    confidence: 0.9
  - pattern: "how should I.*(name|format|structure)"
    confidence: 0.9
  - pattern: "what are the.*(repo|project).*standards"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Everything-Claude-Code-Conventions (The Conventions Layer)

This skill provides the "Conventions Layer" for the CIEL framework, ensuring that all generated code, commits, and project structures adhere to established standards. It is the stylistic "Source of Truth" for CIEL's engineering output.

## Integration Context

Adapted from `~/.agents/skills/everything-claude-code-conventions/`. This skill provides the granular rules used by the **Auditor** sub-agent in the **do** (Execution) phase and the **Verification-Loop** (Finality).

## Core Standards

### 1. Linguistic & Naming
- **Files/Functions**: `camelCase`
- **Classes**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Imports**: Relative imports are preferred.

### 2. Commit Protocol
CIEL MUST use **Conventional Commits**:
- `feat`: New features
- `fix`: Bug fixes
- `test`: Adding or updating tests
- `docs`: Documentation changes
- `chore`: Maintenance tasks
- **Mood**: Use imperative mood ("Add", "Fix", "Update").

### 3. Orchestration Logic
- **Discovery**: During Phase 0 of a plan, CIEL must re-verify these conventions against the current repo state (e.g., checking `package.json` and `.prettierrc`).
- **Audit**: The **Auditor** sub-agent MUST flag any implementation that deviates from these naming or structural patterns.
- **Persistence**: All successful conventions must be indexed in **MemPalace** as `EngineeringStandard` nodes.

## Standard Workflows

### 1. Adding a New Skill
1. Create `skills/{skill-name}/SKILL.md`.
2. Add adaptation metadata if applicable.
3. Index in MemPalace.

### 2. Addressing Review Feedback
1. Explicitly cite the feedback in the commit message.
2. Update the relevant skill/agent/code to resolve the discrepancy.
3. Re-run the **verification-loop**.

---
*Original Architecture and Error Handling patterns preserved at source.*
