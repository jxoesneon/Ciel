---
name: scroll-driven-animation
version: 1.0.0
format: skill/1.0
description: GSAP ScrollTrigger standards for scroll-linked animations, pinning, and parallax.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(scroll|parallax|pin).*(animation|trigger|gsap)"
    confidence: 0.95
  - pattern: "scrolltrigger"
    confidence: 1.0
---

# CIEL ADAPTATION: Scroll-Driven Animation (ScrollTrigger)

This skill formalizes the implementation of scroll-linked effects. it prioritizes layout stability and performant "Scrub" logic.

## ScrollTrigger Core
1. **Targeting**: Attach ScrollTrigger to the **Timeline** or top-level **Tween**, never to children inside a timeline.
2. **Scrub vs Actions**:
   - Use `scrub: true` (or a number like `1`) for direct link between scroll and progress.
   - Use `toggleActions` for discrete triggers (play/pause/reverse).
3. **Pinning**: Don't animate the pinned element itself; animate its children. Set `pinSpacing: true` (default) to preserve layout flow.

## The "Fake" Horizontal Pattern
To implement horizontal scroll inside a vertical page:
1. Pin the container.
2. Animate the horizontal wrapper's `xPercent` to `-100`.
3. **Mandate**: Use `ease: "none"` on the horizontal tween to ensure a 1:1 scroll-to-position mapping.
4. Use `containerAnimation` for any elements that should trigger *during* the horizontal slide.

## Operational Standards
- **Refresh**: Call `ScrollTrigger.refresh()` AFTER dynamic content (images/fonts) loads.
- **Priority**: Use `refreshPriority` if creating triggers in a non-sequential (async) order.
- **Markers**: Set `markers: true` during dev; verify they are REMOVED before integration.

## Anti-Patterns
- **Nesting Triggers**: Putting a `scrollTrigger` inside a tween that is already inside a timeline (breaks logic).
- **Ease Jitter**: Using an ease (like `power2`) on a scrubbed animation (makes it feel disconnected).
- **Amnesiac Refresh**: Forgetting to refresh after an AJAX content load, causing trigger positions to be offset.
