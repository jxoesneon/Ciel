---
name: compose-multiplatform
version: 1.0.0
format: skill/1.0
description: Shared UI patterns for Android, iOS, Desktop, and Web using Compose Multiplatform.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(build|design).*(compose multiplatform|jetpack compose).*(ui|interface)"
    confidence: 0.95
  - pattern: "compose multiplatform patterns"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Compose Multiplatform (Shared UI)

This skill formalizes cross-platform UI development using Compose. It focuses on skippable recomposition and platform-agnostic state management.

## State Management
- **Single Source of Truth**: Use a single data class for screen state (e.g., `ItemListState`).
- **ViewModel**: Expose state as `StateFlow`. Collect in Compose using `collectAsStateWithLifecycle()`.
- **Event Sinks**: Use a sealed interface for screen events (e.g., `ItemListEvent`) to minimize callback lambda count.

## Design Patterns
1. **Slot-Based APIs**: Design components with `@Composable` lambda parameters for maximum flexibility.
2. **Modifier Discipline**: Apply modifiers in order: Layout -> Shape -> Background -> Interaction.
3. **Expect/Actual**: Use for platform-specific UI features (e.g., Status Bar control, native share sheets).

## Performance & Theming
- **Stability**: Mark UI models as `@Stable` or `@Immutable` to enable skippable recomposition.
- **Keys**: Use `key()` in lists to ensure stable item reuse.
- **Material 3**: Enforce dynamic theming using `MaterialTheme` color schemes.

## Anti-Patterns
- **Deep Nav Injection**: Passing `NavController` deep into the tree. Pass lambda callbacks instead.
- **MutableState in VM**: Using `mutableStateOf` in ViewModels (use `StateFlow` for lifecycle safety).
- **Heavy Recomposition**: Performing allocations or filters directly in the `@Composable` function.
