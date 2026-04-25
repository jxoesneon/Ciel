---
name: ciel-swift-and-ios-standards
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Swift 6.2 concurrency, actor-based persistence, and Liquid Glass design.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(design|build|review).*(swift|ios|concurrency|actor|glass)"
    confidence: 0.95
  - pattern: "approachable concurrency"
    confidence: 1.0
---

# CIEL ADAPTATION: Swift & iOS Standards (The Apple Layer)

This skill formalizes modern Swift development. it prioritizes data-race safety and the Liquid Glass design language.

## Approachable Concurrency (Swift 6.2+)
1. **Single-Threaded Default**: Start on `MainActor`. Async functions stay on the calling actor by default.
2. **Isolated Conformances**: Conforming MainActor types to protocols safely without `@Sendable` workarounds.
3. **Explicit Parallelism**: Use `@concurrent` ONLY for CPU-intensive background tasks (e.g., image processing).
4. **Static State**: Protect all global/static variables with `MainActor` or actor isolation.

## Actor-Based Persistence
- **Repository**: Use `actor` for serialized access to local data.
- **In-Memory Cache**: Maintain a dictionary for O(1) lookups; persist to disk atomically on write.
- **Hard Gate**: ALWAYS use `.atomic` writes to prevent file corruption.
- **Initialization**: Load cache synchronously in `init` to avoid async lifecycle complexity.

## Liquid Glass Design (iOS 26+)
- **Container First**: Wrap multiple glass elements in a `GlassEffectContainer` for performance and morphing.
- **Interactivity**: Use `.interactive()` only on touch/pointer targets.
- **Transitions**: Use `@Namespace` + `glassEffectID` for smooth morphing on view hierarchy changes.

## Anti-Patterns
- **Implicit Offloading**: Assuming async functions run in the background (they stay on the caller in 6.2).
- **DispatchQueue Overuse**: Using legacy queues instead of actors for new thread-safe state.
- **Opaque Glass**: Placing opaque backgrounds behind glass elements (destroys the translucency effect).
