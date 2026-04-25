---
name: animation-performance-utils
version: 1.0.0
format: skill/1.0
description: Performance optimization and math utilities for high-performance JS animations.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(optimize|speed up).*(animation|gsap|fps|60fps)"
    confidence: 0.9
  - pattern: "gsap.utils"
    confidence: 1.0
---

# CIEL ADAPTATION: Animation Performance & Utils (Optimization)

This skill formalizes the "Speed Layer" of CIEL animations. it uses mathematical utilities to reduce overhead and visual jitter.

## Performance Heuristics
1. **will-change**: Apply `will-change: transform` in CSS to elements that will animate frequently.
2. **Batching**: Use `gsap.quickTo()` for mouse-followers or high-frequency updates (60fps) to reuse a single tween.
3. **Stagger over Loop**: Use GSAP's native `stagger` instead of `map()` or `forEach()` to create individual tweens.
4. **Virtualization**: For lists > 100 items, only animate items currently in the viewport.

## The Utils Library (Math)
- **Resolution Control**: Use `clamp(min, max)` to bound user-driven values.
- **Mapping**: Use `mapRange(inMin, inMax, outMin, outMax)` to convert scroll progress (0-1) to values (e.g., 0-360 deg).
- **Function Form**: Omit the `value` argument to get a reusable transformer function: `const mapper = mapRange(0, 1, 0, 100);`.
- **Arrays**: Use `toArray()` to normalize selectors, NodeLists, and elements into a true JS array.

## Operational Standards
- **Throttling**: Use `Observer` or `gsap.ticker` for input handling instead of raw `mousemove` events.
- **Snap**: Use `snap()` for grid-aligned or step-based movements.

## Anti-Patterns
- **Layout Thrashing**: Mixing DOM reads (`offsetWidth`) and GSAP writes in a tight loop.
- **Global Ticker Overload**: Adding 20+ listeners to the `gsap.ticker` without cleaning them up.
- **Heavy Math in Body**: Performing complex trigonometry inside a `body` loop instead of using pre-calculated `mapRange` lookups.
