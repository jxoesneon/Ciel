---
name: perl-development
version: 1.0.0
format: skill/1.0
description: Modern Perl 5.36+ idioms, taint-aware security, and Test2::V0 testing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|review).*(perl|cpan|cpanfile)"
    confidence: 0.95
  - pattern: "modern perl"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Perl Development (Modern & Secure)

This skill formalizes the use of Perl 5.36+ for CIEL development. it prioritizes the modern feature set (signatures, postfix deref) and defensive security.

## Modern Standards (v5.36+)
1. **The Preamble**: Use `use v5.36;` to enable strict, warnings, and subroutine signatures.
2. **Signatures**: Use native signatures for all subroutines. Prohibit manual `@_` unpacking.
3. **Dereferencing**: Use postfix syntax (`$ref->@*`, `$ref->%*`) for readability.
4. **OO**: Prefer **Moo** for lightweight, type-safe objects. Use **Moo::Role** for composition.

## Defensive Security
- **Taint Mode**: Enable `-T` for all web/CGI entry points. Explicitly untaint via specific regex.
- **Injection**: Mandate **DBI placeholders** (`?`) for all SQL. Prohibit string interpolation in queries.
- **File I/O**: ALWAYS use the 3-argument `open` with explicit encoding (e.g., `<:encoding(UTF-8)`).

## Testing (Test2::V0)
- **Framework**: Standardize on **Test2::V0**. it is CIEL's canonical Perl testing suite.
- **Deep Comparison**: Use the hash/array builders (`field`, `item`) for partial structure matching.
- **Mocking**: Use `Test::MockModule` to isolate boundaries without leaking state between tests.

## Anti-Patterns
- **Global Pollution**: Using `our` variables for cross-module state. Use DI or attributes.
- **Two-Arg Open**: Using `open FH, $file` (shell injection risk).
- **String Eval**: Using `eval "require $m"` (remote code execution risk).
