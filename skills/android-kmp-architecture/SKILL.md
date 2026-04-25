---
name: android-kmp-architecture
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Android and Kotlin Multiplatform (KMP) architecture, module boundaries, and dependency inversion.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(structure|setup|architect).*(android|kmp|kotlin multiplatform).*(project|module)"
    confidence: 0.9
  - pattern: "android clean architecture"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Android & KMP Architecture (Structural Layer)

This skill formalizes the "Structural Layer" for Android and KMP projects. It enforces module isolation and dependency inversion to ensure cross-platform portability and testability.

## Recommended Module Layout
```
project/
├── app/                  # Platform entry point (Android), DI wiring.
├── domain/               # Pure Kotlin. UseCases, Domain Models, Repository Interfaces.
├── data/                 # Repository Impls, DataSources (Room/SQLDelight), Network (Ktor).
├── presentation/         # ViewModels, UI State, Flow collection.
└── core/                 # Shared utilities, error types, base classes.
```

## Dependency Rules (Immutable)
1. **Domain Isolation**: `domain` MUST NEVER depend on `data`, `presentation`, or any platform framework.
2. **One-Way Flow**: `app` -> `presentation` -> `domain` <- `data`.
3. **Mappers**: Keep mappers (e.g., `Entity -> Domain`) as extension functions in the `data` layer.

## Core Patterns
- **UseCase**: Single business operation. Use `operator fun invoke` for clean call sites.
- **Result/Try**: Use a sealed interface (e.g., `Result<T>`) for all repository/use-case returns to avoid unchecked exceptions.
- **DI**: Standardize on Koin (KMP-friendly) or Hilt (Android-only).

## Anti-Patterns
- **Context Leaking**: Passing Android `Context` or framework classes into the `domain` layer.
- **Fat ViewModels**: Putting business logic in ViewModels instead of UseCases.
- **Circular Deps**: Allowing `domain` to know about `data` implementations.
