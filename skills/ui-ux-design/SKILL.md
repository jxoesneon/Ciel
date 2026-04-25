---
name: ui-ux-design
version: 1.0.0
format: skill/1.0
description: CIEL's design intelligence for web and mobile. Provides style selection, UX guidelines, and accessibility standards.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|refactor).*(ui|ux|interface|page|component)"
    confidence: 0.95
  - pattern: "recommend (style|color|font)"
    confidence: 0.9
---

# CIEL ADAPTATION: UI/UX Design (Intelligence)

This skill formalizes visual design and user experience quality control. It provides the "Intelligence" needed to build professional, accessible, and high-performance interfaces.

## The Design Priority Matrix
1. **Accessibility (CRITICAL)**: 4.5:1 contrast, Alt text, Keyboard nav, 44x44pt touch targets.
2. **Performance (HIGH)**: WebP/AVIF, Lazy loading, CLS < 0.1, 16ms frame budget.
3. **Style Selection (HIGH)**: Match product type (Minimal, Glassmorphism, Brutalism). Consistent SVG icons.
4. **Layout (MEDIUM)**: Mobile-first breakpoints, 8pt spacing rhythm, line-length control (65-75 chars).
5. **Animation (LOW)**: 150-300ms duration, spring physics, motion conveys meaning.

## Operational Workflow
### 1. Requirements Extraction
Identify the **Product Type** (Tool, Social, Fintech) and **Target Audience** (B2B, C-end).

### 2. Design System Generation
The Orchestrator MUST define a Master Design System before coding:
- **Pattern**: Landing, Dashboard, Admin.
- **Style**: minimalism, dark mode, content-first.
- **Tokens**: Semantic color tokens (Primary, Error, Surface) NOT raw hex.

### 3. Persistance (The Master/Override Pattern)
Save the design system to `docs/design/MASTER.md`. For specific pages, create `docs/design/pages/[name].md` to document deviations.

## Anti-Patterns
- **Emoji-Driven Design**: Using emojis as structural navigation icons.
- **Gray-on-Gray**: Using low-contrast text that violates WCAG AA standards.
- **Instant Snaps**: Having UI elements appear/disappear without subtle transitions.
- **Orphaned Selectors**: Hardcoding per-screen styling instead of using theme tokens.
