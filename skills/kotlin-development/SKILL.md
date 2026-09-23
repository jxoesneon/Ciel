---
name: kotlin-development
description: Idiomatic Kotlin patterns, structured concurrency (Coroutines/Flow), and Kotest/MockK testing.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Kotlin Development (Idiomatic & Async)

This skill formalizes the "Modern JVM" layer. it prioritizes null safety, expression-oriented code, and structured concurrency.

## Idiomatic Standards

1. **Null Safety**: Leverage `?.` and `?:`. Prohibit `!!`.
2. **Type System**: Use `sealed class/interface` for exhaustive state modeling. Use `value class` for zero-overhead wrappers.
3. **Expressions**: Favor expression bodies (`fun x() = ...`) and `when` as an expression.
4. **Scope Functions**: Use `apply` for config, `let` for transformation, `also` for side effects.

## Structured Concurrency (Async Layer)

- **Scope**: Never use `GlobalScope`. Use `viewModelScope`, `lifecycleScope`, or `coroutineScope {}`.
- **Parallelism**: Use `async { ... }.await()` for concurrent data loading.
- **Reactivity**:
  - **StateFlow**: For UI state (use `WhileSubscribed(5000)`).
  - **SharedFlow**: For one-time effects (Snackbars, Navigation).
- **Dispatchers**: `Default` for CPU, `IO` for I/O, `Main` for UI.

## Testing (Kotest & MockK)

- **Spec Style**: Standardize on `BehaviorSpec` (Given/When/Then) for complex logic; `StringSpec` for units.
- **Mocking**: Use `coEvery` and `coVerify` for suspend functions.
- **Flow Testing**: Use `turbine` to test Flow emissions.

## Anti-Patterns

- **Platform Leakage**: Passing Java-style `null` pointers without checking at boundaries.
- **Thread.sleep**: Using sleep in coroutine tests (use `advanceTimeBy`).
- **Fat Composable**: Performing heavy Flow creation/logic directly in UI functions without `remember`.
