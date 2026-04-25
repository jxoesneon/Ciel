---
name: context-and-memory-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for project context, voice modeling, and codebase walkthroughs.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(save|resume|model|tour).*(context|voice|memory|walkthrough)"
    confidence: 0.95
  - pattern: "context keeper"
    confidence: 1.0
---

# CIEL ADAPTATION: Context & Memory (Persistence)

This skill formalizes the management of project-specific knowledge and stylistic identity. it prioritizes high-fidelity recall and guided onboarding.

## Context Keeper (ck)
1. **Session State**: Use `/ck:save` at the end of every high-value session.
2. **Briefing**: Use `/ck:resume` at start to load goals, decisions, and blockers.
3. **Consistency**: Ensure `context.json` is the single source of truth for project state.

## Brand Voice Modeling
- **Extraction**: Gather 5-20 original samples (X posts, memos, code comments).
- **Priorities**: Mechanisms and receipts beat adjectives. Direct, concrete, and compressed.
- **Hard Bans**: Delete "fake curiosity," "excited to share," and LinkedIn-style "thought leader" cadence.

## Code Tours (`.tour`)
- **Persona Target**: Write for specific readers (`new-joiner`, `security-reviewer`, `architect`).
- **Anchors**: Use real file paths and line numbers. Prefer pattern-based anchors for volatile files.
- **SMIG Format**: Description must answer: Situation, Mechanism, Implication, and Gotcha.

## Anti-Patterns
- **Amnesiac Start**: Starting a new session without resuming the previous context.
- **Hallucinated Anchors**: Committing a `.tour` file with guessed line numbers.
- **Vibe Modeling**: Deriving a voice from 2 samples instead of a representative set.
