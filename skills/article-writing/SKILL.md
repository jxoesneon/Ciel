---
name: article-writing
version: 1.0.0
format: skill/1.0
description: Voice-aligned long-form content creation (blog posts, tutorials, newsletters).
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(write|draft|polished).*(article|blog|essay|newsletter|tutorial)"
    confidence: 0.9
  - pattern: "write a guide for"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Article Writing (The Content Layer)

This skill formalizes the creation of long-form content that avoids the "AI Smoothing" trap. it prioritizes proof, artifacts, and a distinctive point of view.

## Core Mandates
1. **Example First**: Lead with code, a screenshot, or a number. Explain AFTER the example.
2. **Tight Sentences**: Eliminate throat-clearing bridges (e.g., "In today's landscape").
3. **Voice Reuse**: Pull the `VOICE PROFILE` from the `brand-identity` skill.

## Writing Pipeline
- **Outline**: Build a hard outline where every section has a specific "Job."
- **Draft**: Use the "Sharp Operator" voice—concrete, unsentimental, and useful.
- **Audit**: Run an Auditor sub-agent to find and delete "game-changer" adjectives.

## Quality Gate
- [ ] Factual claims are backed by source files or research.
- [ ] No generic AI transitions.
- [ ] Voice matches the project's Master Guidelines.
- [ ] Formatting is medium-specific (Markdown for GitHub, HTML for Newsletters).

## Anti-Patterns
- **Throat-Clearing**: Writing 3 paragraphs of introduction before getting to the value.
- **Adjective Bloat**: Using "revolutionary" instead of showing the actual performance metrics.
- **Fake Vulnerability**: Adding manufactured "founder struggles" for engagement juice.
