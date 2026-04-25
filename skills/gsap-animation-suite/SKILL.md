---
name: gsap-animation-suite
version: 1.0.0
format: skill/1.0
description: CIEL's unified framework for high-performance GSAP animations, timelines, and plugins.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(animate|build).*(gsap|timeline|tween|ease|svg|morph)"
    confidence: 0.9
  - pattern: "gsap.to"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: GSAP Animation Suite (The Motion Layer)

This skill manages complex animation sequencing and visual polish. it prioritizes performance-safe transforms over layout-heavy properties.

## The Motion Mandates
1. **Compositor First**: Animate `x`, `y`, `rotation`, and `opacity`. Prohibit animating `width`, `height`, `top`, or `left` unless layout reflow is explicitly required.
2. **Timeline Discipline**: Prefer `gsap.timeline()` for multi-step sequences. Use the **Position Parameter** (`"<"`, `"+=0.2"`) instead of chaining `delay`.
3. **Plugin Registration**: Register every plugin (Flip, Draggable, MorphSVG) ONCE at the application root.

## Architecture & Sequencing
- **Defaults**: Set `gsap.defaults({ duration: 0.5, ease: "power2.out" })` to ensure consistent "feel."
- **Nesting**: Break complex "Hero" animations into nested timelines for maintainability.
- **Overwrite**: Use `overwrite: "auto"` to prevent conflicting animations from fighting over the same property.

## Visual Integrity (SVG)
- **DrawSVG**: Use for "path writing" effects. requires `stroke-width` in CSS.
- **MorphSVG**: Use for icon transitions. Convert primitives to paths via `MorphSVGPlugin.convertToPath()` early.
- **Origins**: Use `svgOrigin` for rotation around a global SVG point; `transformOrigin` for local center.

## Anti-Patterns
- **Delay Chaining**: Building a sequence by manually calculating `delay: 1.5`, `delay: 2.0` (brittle).
- **Invisible Tweens**: Failing to use `autoAlpha: 0` for hidden elements, leaving them clickable.
- **Jittery CSS**: Mixing GSAP transforms with CSS Transitions on the same element.
