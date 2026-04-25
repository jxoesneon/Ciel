---
name: go-development
version: 1.0.0
format: skill/1.0
description: CIEL's framework for idiomatic Go development, table-driven testing, and performance benchmarking.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(design|build|review).*(go|golang|goroutine)"
    confidence: 0.95
  - pattern: "go test"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Go Development (Simplicity & Concurrency)

This skill formalizes the development of performant, readable Go services. it prioritizes the "Errors are Values" philosophy and idiomatic concurrency.

## Idiomatic Standards
1. **Zero Value Utility**: Design types so their zero value is ready to use (e.g., `sync.Mutex`, `bytes.Buffer`).
2. **Interface Discipline**: "Accept interfaces, return structs." Define interfaces where they are used (consumer-side).
3. **Error Handling**: ALWAYS check errors immediately. Wrap with context: `fmt.Errorf("do X: %w", err)`.
4. **Returning Early**: Handle errors first; keep the "happy path" unindented.

## Concurrency Mandates
- **Context First**: Pass `context.Context` as the first argument to all long-running or I/O-bound functions.
- **Goroutine Hygiene**: Never start a goroutine without knowing how it will stop. Use `errgroup` for coordination.
- **Channel Safety**: Prefer communication by channels over shared memory. Use buffered channels only when necessary for performance.

## TDD & Performance (testing)
- **Table-Driven Tests**: Use anonymous structs for test cases.
- **Benchmarks**: Mandate `BenchmarkX(b *testing.B)` for all performance-critical hot paths.
- **Fuzzing (1.18+)**: Use `f.Fuzz` for complex parsers or input validation logic.
- **Race Detector**: ALWAYS run tests with `-race` in CI.

## Anti-Patterns
- **Panic for Control Flow**: Using `panic` instead of returning an `error`.
- **Global Mutable State**: Initializing DB or Loggers in `init()` instead of using dependency injection.
- **Naked Returns**: Using unnamed return variables in functions longer than 10 lines.
