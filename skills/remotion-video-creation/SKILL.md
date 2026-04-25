---
name: remotion-video-creation
version: 1.0.0
format: skill/1.0
description: Best practices for programmable video creation using Remotion and React.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(create|build|code).*(video|animation).*(remotion)"

    confidence: 0.9

  - pattern: "remotion"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Remotion Video (Programmable Media)

This skill formalizes the use of Remotion for creating videos through code. It is used for overlays, data visualizations, and repeatable motion graphics.

## Architecture Guidelines

- **Composition Standard**: Define fixed dimensions and frame rates in `calculate-metadata.md`.
- **Asset Isolation**: Import images, fonts, and videos as isolated React components (`Img`, `Video`).
- **Sequencing**: Use `Sequence` and `Series` for temporal control. Avoid raw `setTimeout` or `setInterval`.

## Animation Principles

- **Interpolation**: Use `interpolate()` with easing or spring curves for all property changes.
- **Shared Elements**: Maintain visual continuity through shared element transitions across scenes.
- **Dynamic Captions**: Implement TikTok-style captions with word highlighting via `@remotion/captions`.

## Performance

- **Transformation-Only**: Animate `transform` and `opacity` to avoid layout reflows.
- **Browser Decoding**: Use `can-decode.md` to check asset compatibility before rendering.

## Anti-Patterns

- **State-Heavy Compositions**: Triggering heavy business logic during a render frame.
- **Fixed Durations**: Hardcoding second-based durations instead of frame-based calculations.
- **CSS-Transitions**: Using raw CSS transitions instead of Remotion's `interpolate` (breaks deterministic rendering).
