---
name: ciel-meta-and-utility
version: 1.0.0
format: skill/1.0
description: CIEL's framework for skill discovery, worktrees, and capability auditing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(manage|audit|configure).*(skill|worktree|workspace|visa)"

    confidence: 0.9

  - pattern: "using git worktrees"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Meta & Utility (The Orchestrator)

This skill formalizes the self-management of CIEL and specialized utility workflows.

## Skill Discovery (Superpowers)

1. **Hard Mandate**: If there is a 1% chance a skill applies, you MUST invoke it.
2. **Priority**: Process skills (Brainstorming/Debugging) > Implementation skills.
3. **Integrity**: Follow Rigid skills (TDD) exactly; adapt Flexible skills (Patterns) to context.

## Workspace Isolation (Worktrees)

- **Directory**: Follow priority: Existing `.worktrees/` -> `CLAUDE.md` preference -> Ask user.
- **Safety (Hard Gate)**: Verify the directory is in `.gitignore` before creation. Fix immediately if not.
- **Baseline**: Run tests in the fresh worktree to ensure a clean start before implementing.

## Workspace Surface Audit

- **Inventory**: Identify configured MCPs, environment keys, and installed skills.
- **Parity Check**: Compare workspace capabilities against official benchmarks.
- **Next Moves**: Propose concrete additions (Skills, Hooks, Agents) ordered by business impact.

## Visa Doc Intelligence

- **Workflow**: Conversion -> Rotation -> OCR -> Translation -> PDF.
- **OCR Hierarchy**: macOS Vision -> EasyOCR -> Tesseract.
- **Integrity**: Preserve original structure and names. Annotate as "Certified English translation."

## Anti-Patterns

- **Worktree Pollution**: Committing worktree metadata to the main repository.
- **Amnesiac Audit**: Summarizing "How" to install instead of "What" is actually missing.
- **Silent Skill Skipping**: Skipping a relevant skill because you "remembered the logic."
