---
name: cpp-development
version: 1.0.0
format: skill/1.0
description: Modern C++ (17/20+) standards based on Core Guidelines, RAII, and GoogleTest.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: '(design|build|review).*(cpp|c\+\+|cmake|gtest)'
    confidence: 0.95
  - pattern: 'cpp core guidelines'
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: C++ Development (Core Guidelines)

This skill formalizes the use of modern C++ (C++17/20+). it prioritizes resource safety (RAII) and the C++ Core Guidelines.

## The Integrity Layer (RAII)
1. **No Raw Ownership**: Never transfer ownership via raw pointers (`T*`). Use `std::unique_ptr` or `std::shared_ptr`.
2. **Resource Binding**: Bind all resource lifetimes (Files, Locks, Sockets) to object lifetimes via RAII classes.
3. **Smart Allocation**: Avoid `new` and `delete`. Use `std::make_unique` and `std::make_shared`.

## Architectural Standards
- **Immutability**: Declaring objects `const` or `constexpr` by default. Member functions must be `const` unless they mutate state.
- **Type Safety**: Use `enum class` over plain `enum`. Prohibit C-style casts; use `static_cast` or `dynamic_cast`.
- **Interfaces**: Pass "in" parameters by value for cheap types and `const&` for expensive types.

## TDD & Verification
- **Framework**: **GoogleTest** (gtest) + **GoogleMock** (gmock).
- **CMake/CTest**: Use `gtest_discover_tests()` for stable discovery. Run with `--output-on-failure`.
- **Sanitizers**: ALWAYS run tests with **AddressSanitizer (ASan)** and **UndefinedBehaviorSanitizer (UBSan)** in CI.

## Anti-Patterns
- **Naked New**: Calling `new` without immediately wrapping it in a smart pointer or RAII class.
- **Global Variables**: Using non-const globals (violates I.2).
- **Implicit Conversions**: Allowing narrowing/lossy arithmetic conversions without explicit casting.
