---
name: rules-distill
version: 1.0.0
format: skill/1.0
description: The protocol for extracting cross-cutting principles from skills and distilling them into global workspace rules.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(distill|extract|update).*(rules|principles)"
    confidence: 0.9
  - pattern: "turn (skills|patterns) into rules"
    confidence: 0.95
---

# CIEL ADAPTATION: Rules Distill (Principle Extraction)

This skill formalizes the mechanism by which CIEL evolves its global ruleset. It applies the "Deterministic Collection + LLM Judgment" principle to identify cross-cutting behaviors that should be promoted from individual skills to mandatory rules.

## The Distillation Workflow

### Phase 1: Inventory (Deterministic)
The system exhaustively scans the installed skill registry (`Ciel/skills/`) and the current ruleset to create a baseline inventory of principles.

### Phase 2: Cross-Read & Match (Judgment)
An Auditor sub-agent cross-references all skills to find principles that meet the **Promotion Criteria**:
1. **Ubiquity**: The principle appears in 2+ distinct skills.
2. **Actionability**: It can be written as a "Do X" or "Don't do Y" instruction.
3. **Risk**: There is a clear violation risk if the principle is ignored.
4. **Novelty**: It is not already sufficiently covered in the existing rules.

### Phase 3: Review & Execution
Proposed rule updates (Append, Revise, or New File) are presented to the user for explicit approval. **Manual approval is mandatory; CIEL never modifies its own ruleset automatically.**

## Output Format
Each proposal must include:
- **Principle**: The exact rule text.
- **Evidence**: List of skills where the principle was observed.
- **Violation Risk**: What happens if this rule is broken.
- **Verdict**: Append | Revise | New Section | New File.

## Anti-Patterns
- **Over-Abstraction**: Creating vague rules (e.g., "Code should be clean") that are not actionable.
- **Skill Leakage**: Moving language-specific hacks that belong in a skill into the global ruleset.
- **Automated Refactoring**: Modifying rules without a user-in-the-loop review.
