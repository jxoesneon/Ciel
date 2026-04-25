---
name: animation-framework-integration
version: 1.0.0
format: skill/1.0
description: Standards for integrating GSAP into React, Vue, and Svelte while ensuring clean memory management.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(gsap|animation).*(react|vue|svelte|mounted|cleanup)"

    confidence: 0.9

  - pattern: "useGSAP"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Animation Framework Integration (Memory & Lifecycle)

This skill dictates how to safely use GSAP inside component-based frameworks. it focuses on the "Mounted/Unmounted" lifecycle and selective scoping.

## React Integration (@gsap/react)

- **Mandate**: Use the `useGSAP()` hook. it handles `gsap.context()` and `revert()` automatically.
- **Scoping**: Always pass a `scope` ref (e.g., `containerRef`) so selectors like `.box` are limited to the component.
- **Event Safety**: Wrap handlers (e.g., `onClick`) in `contextSafe()` to ensure they are cleaned up on unmount.

## Vue & Svelte Integration

1. **Mounted**: Create all tweens/triggers inside `onMounted()` (Vue) or `onMount()` (Svelte).
2. **Context**: Use `gsap.context(() => { ... }, containerRef)` to group all animations in the component.
3. **Cleanup (Hard Gate)**: Call `ctx.revert()` in `onUnmounted()` / the cleanup return. PROHIBIT un-cleaned animations.

## Scoping Mandate

Selector strings (e.g., `".box"`) MUST NEVER be used without a scope ref. This prevents a component from accidentally animating elements in other instances or the rest of the page.

## SSR Guardrails

- **Client-Only**: GSAP and ScrollTrigger MUST only execute on the client.
- **Guard**: Wrap GSAP logic in `if (import.meta.client)` or check for `typeof window !== 'undefined'`.

## Anti-Patterns

- **The Selector Leak**: Animating `".active"` and accidentally moving every active button on the entire site.
- **The Memory Leak**: Forgetting to `revert()` a ScrollTrigger, causing it to keep calculating positions after the page changes.
- **init() Logic**: Creating animations in the component constructor or setup before the DOM nodes exist.
