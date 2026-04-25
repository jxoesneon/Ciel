---
name: python-development
version: 1.0.0
format: skill/1.0
description: Idiomatic Python patterns, modern type hints, and pytest-driven development.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(design|build|review).*(python|pip|pytest|type hint)"

    confidence: 0.9

  - pattern: "idiomatic python"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Python Development (Idiomatic & Typed)

This skill formalizes the "Modern Python" layer. it prioritizes readability (Zen of Python), explicit type hints, and EAFP error handling.

## Idiomatic Standards

1. **Zen Compliance**: Readability over cleverness. Explicit over implicit.
2. **Type Hints (3.9+)**: Annotate all public functions using built-in generics (e.g., `list[str]`).
3. **Error Handling**: Follow **EAFP** (Easier to Ask Forgiveness). Use `try/except` for expected errors.
4. **Resource Management**: ALWAYS use `with` statements (Context Managers) for I/O.

## Modern Patterns

- **Data Containers**: Use `@dataclass` for mutable state and `NamedTuple` for immutable state.
- **Lazy Eval**: Prefer Generator Expressions over large List Comprehensions for memory efficiency.
- **Validation**: Use **Pydantic** (v2) for strict runtime data validation and parsing.

## Testing (pytest)

- **TDD**: Write failing tests first. 80% coverage target.
- **Fixtures**: Use `yield` fixtures for setup/teardown. Avoid manual cleanup logic in tests.
- **Mocking**: Use `@patch` for external I/O. Prefer `autospec=True` to catch API misuse.

## Anti-Patterns

- **Mutable Defaults**: Using `def foo(x=[])`. Use `None` and initialize inside the function.
- **Bare Except**: Catching all exceptions (`except:`) instead of specific classes.
- **Clever Comprehensions**: Writing 3-line list comprehensions that are unreadable.
