---
name: flutter-development
version: 1.0.0
format: skill/1.0
description: Production-ready Dart and Flutter patterns, state management, and code review standards.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(build|review|refactor).*(flutter|dart).*(feature|widget|code)"

    confidence: 0.9

  - pattern: "flutter patterns"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Flutter Development (Patterns & Review)

This skill formalizes the development and review of Flutter/Dart applications. It combines idiomatic implementation patterns with a rigorous quality checklist.

## Implementation Standards

1. **Null Safety**: Avoid `!`. Prefer `?.`, `??`, or Dart 3 pattern matching (`switch (user) { ... }`).
2. **Immutability**: Use sealed classes for state hierarchies and `freezed` for boilerplate-free `copyWith`.
3. **Widget Purity**: Extract sub-widgets to Classes, NOT private methods (enables `const` propagation and element reuse).
4. **Reactivity**: check `mounted` after every `await` before using `BuildContext`.

## State Management Decision Matrix

- **Simple/Atomic**: `ValueNotifier` or `Provider`.
- **Complex/Async**: `BLoC/Cubit` (Event-driven) or `Riverpod` (Functional/Derived).
- **Hard Gate**: Never store `BuildContext` in a state manager or singleton.

## Quality Checklist (The Auditor Gate)

- [ ] No `print()` in production; use `log()`.
- [ ] Widgets under 100 lines.
- [ ] Colors/Fonts from `Theme.of(context)`, never hardcoded hex.
- [ ] `const` constructors used at every possible boundary.

## Anti-Patterns

- **Boolean Flag Soup**: Using `isLoading`, `isError` flags instead of a sealed `AsyncState` type.
- **Deep Nesting**: Nesting more than 5 levels of widgets (Extract to sub-classes).
- **Bang-Operator Abuse**: Using `!` to "fix" nullability errors instead of proper guarding.
