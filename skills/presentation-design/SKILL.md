---
name: presentation-design
version: 1.0.0
format: skill/1.0
description: Create zero-dependency, animation-rich HTML presentations and pitch decks from specs or PPTX.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(create|build).*(presentation|deck|slides|pitch)"
    confidence: 0.95
  - pattern: "convert (pptx|powerpoint) to (html|slides)"
    confidence: 1.0
---

# CIEL ADAPTATION: Presentation Design (The Deck Layer)

This skill formalizes the creation of "Frontend Slides"—strategic, zero-dependency HTML presentations that run in any browser.

## The Presentation Core
- **Zero Dependencies**: Single HTML file with inline CSS/JS.
- **Viewport Fit**: Hard gate—no scrolling inside slides. Use `100dvh` and `clamp()`.
- **Storytelling**: Use copywriting formulas (e.g., Problem/Solution) to structure content.

## Workflow
### 1. Visual Exploration (Show, Don't Tell)
The Orchestrator MUST generate 3 single-slide "Previews" in `.ciel/design/previews/` representing different moods (Focused, Inspired, Energized).

### 2. PPTX Conversion
- **Tool**: Use `python-pptx` to extract raw text and assets.
- **Rule**: Preserve speaker notes and slide order, then apply the "Frontend Slides" styling.

### 3. Navigation Controller
Every deck MUST include a JS class handling:
- Arrow keys / Spacebar.
- Mouse wheel / Touch swipe.
- Intersection Observer reveal animations.

## Content Density Limits
- **Title**: 1 Heading + Subtitle.
- **Content**: Max 6 bullets.
- **Code**: Max 10 lines (use multiple slides for long snippets).

## Anti-Patterns
- **Scroll-Hell**: Creating slides that overflow the viewport.
- **Bullet-Walls**: Putting 20 bullet points on a single slide.
- **Template-Vibes**: Using generic gradients with no project-specific visual identity.
