---
name: swiftui-patterns
version: 1.0.0
format: skill/1.0
description: Modern SwiftUI architecture, state management with @Observable, and view composition best practices.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(build|design).*(swiftui|ios|macos).*(view|interface)"

    confidence: 0.9

  - pattern: "swiftui patterns"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: SwiftUI Patterns (Declarative UI)

This skill formalizes the use of the Observation framework and type-safe navigation in modern SwiftUI.

## State Management (The Modern Way)

- **Shared Model**: Use `@Observable` classes. Replace `ObservableObject` and `@Published`.
- **ViewModel**: Create view models as `@State` in the parent view that owns the data.
- **Binding**: Use `@Bindable` for two-way bindings to `@Observable` properties.
- **Environment**: Use `.environment(object)` to inject shared dependencies.

## View Composition

1. **Fine-Grained Invalidation**: Break views into small structs. SwiftUI only re-renders the subview reading the changed property.
2. **ViewModifier**: Encapsulate reusable styling (e.g., `.cardStyle()`) in modifiers.
3. **Type-Safe Navigation**: Use `NavigationStack` with a `Hashable` destination enum and `NavigationPath`.

## Performance Mandates

- **Lazy Containers**: Use `LazyVStack` and `LazyHStack` for large collections.
- **Stable IDs**: Always use stable, unique IDs in `ForEach` (avoid array indices).
- **No I/O in Body**: Never perform heavy computation or network calls inside the `body` property. Use `.task {}`.

## Anti-Patterns

- **AnyView Abuse**: Using `AnyView` instead of `@ViewBuilder` or `Group`.
- **Async Drift**: Putting async work in `init()` instead of `.task {}`.
- **ObservableObject Legacy**: Using `ObservableObject` in new iOS 17+ codebases.
