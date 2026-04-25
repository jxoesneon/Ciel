---
name: ciel-skill-engineering
version: 1.0.0
format: skill/1.0
description: CIEL's meta-framework for skill creation, writing standards, and self-reproduction.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(create|edit|review).*(skill|persona|soul|reproduce)"
    confidence: 1.0
  - pattern: "node init_skill.cjs"
    confidence: 1.0
---

# CIEL ADAPTATION: Skill Engineering (The Reproduction Layer)

This skill formalizes the creation, packaging, and iteration of CIEL skills. it mandates a TDD-based documentation approach.

## The Skill Lifecycle
1. **Initialize**: Use `init_skill.cjs` to generate a structured template (scripts/refs/assets).
2. **Edit (TDD)**:
   - **RED**: Identify a task where an agent fails/hallucinates without procedural guidance.
   - **GREEN**: Write imperative instructions in `SKILL.md` to bridge that gap.
   - **REFACTOR**: Move detailed references to `references/` to keep the primary skill lean.
3. **Package**: Run `package_skill.cjs` to validate YAML and ensure "No Placeholders."
4. **Install**: Deploy to `--scope workspace` (repo-specific) or `--scope user` (global).

## Design Principles
- **Concise Metadata**: The description MUST specify "When to use." Summary of "How" is prohibited in frontmatter.
- **Progressive Disclosure**: Keep `SKILL.md` < 500 lines. Use linked `.md` files for examples and deep schemas.
- **Degrees of Freedom**: Use high-freedom (text) for heuristics; low-freedom (scripts) for fragile sequences.

## Resource Bundle Patterns
- **Scripts**: For deterministic reliability (e.g., PDF rotation). Must output agent-friendly JSON/STDOUT.
- **References**: For schemas/docs that should be loaded ONLY when needed.
- **Assets**: Templates/Icons that are copied into the output, not read into context.

## Anti-Patterns
- **Placeholder Rot**: Committing a skill with `// Refine implementation logic to align with Ciel 1.0 standards.` or `...` markers.
- **Amnesiac Description**: Describing the tool's history instead of its triggering context.
- **Excessive Freedom**: Using text instructions for a complex API sequence that requires a script.
