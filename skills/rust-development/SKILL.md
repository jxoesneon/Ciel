---
name: rust-development
version: 1.0.0
format: skill/1.0
description: CIEL's framework for idiomatic Rust, zero-cost abstractions, and robust error handling.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(design|build|review).*(rust|cargo|crate|borrow checker)"
    confidence: 0.9
  - pattern: "rust patterns"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Rust Development (Safety & Performance)

This skill formalizes the development of high-integrity Rust applications. it prioritizes ownership-aware design and making illegal states unrepresentable.

## Idiomatic Standards
1. **Ownership & Borrowing**: Pass `&T` (borrow) by default. Clone ONLY when ownership is required and cannot be moved.
2. **Error Handling**:
   - **Apps**: Use `anyhow` for flexible error propagation with context.
   - **Libraries**: Use `thiserror` for structured, typed error variants.
   - **Hard Gate**: Prohibit `.unwrap()` or `.expect()` in production/library code. Use `?` or `unwrap_or`.
3. **Type-Driven Design**: Use `enum` and exhaustive matching to eliminate illegal states. Use the "Newtype" pattern for primitive safety.

## Architecture & Traits
- **Trait Bounds**: Use `impl Trait` for inputs. Use `Box<dyn Trait>` only when dynamic dispatch is required.
- **Async (Tokio)**: Enforce `tokio::spawn` for concurrent tasks. Never block an async executor with `std::thread::sleep`.
- **Visibility**: Expose minimally. Prefer `pub(crate)` over `pub`.

## TDD & Verification
- **Doc Tests**: Include executable examples in doc comments for all public APIs.
- **Property-Based Testing**: Use `proptest` for input validation and complex state transitions.
- **LLVM Cov**: Target **80%+ coverage** using `cargo llvm-cov`.

## Anti-Patterns
- **Wildcard Matching**: Using `_ => {}` in business-critical enums (hides new variants).
- **String over &str**: Using `String` for read-only function parameters.
- **Unsafe Without Reason**: Using `unsafe` without a `// SAFETY:` comment explaining the invariant.
