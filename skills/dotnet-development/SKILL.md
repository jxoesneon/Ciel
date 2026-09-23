---
name: dotnet-development
description: CIEL's framework for modern .NET/C# development, dependency injection, and xUnit testing.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: .NET Development (Modern & Typed)

This skill formalizes the development of robust C# applications using .NET 8+. it prioritizes immutability, explicit DI, and async efficiency.

## Idiomatic Standards

1. **Immutability First**: Use `public sealed record` for all DTOs and Value Objects. Use `init` setters for optional properties.
2. **Explicit Nullability**: Enable `<Nullable>enable</Nullable>`. Use `?` for optional types.
3. **Async Efficiency**: Use `await` all the way down. ALWAYS pass `CancellationToken` to async I/O calls.
4. **DI Discipline**: Depend on abstractions (`IInterface`). Prohibit manual `new Service()` in business logic.

## ASP.NET Core Patterns

- **Minimal APIs**: Use `MapGroup` to organize routes. Use `TypedResults` for unit-testable responses.
- **Options Pattern**: Map config sections to `sealed class` options. Inject via `IOptions<T>`.
- **Middleware**: Use for cross-cutting concerns (Logging, Auth, Performance).

## TDD & Verification

- **Framework**: **xUnit** + **FluentAssertions**.
- **Mocking**: Use **NSubstitute** for clean, interface-based mocking.
- **Integration**: Use **WebApplicationFactory** and **Testcontainers** for real infrastructure testing (SQL, Redis).
- **Gate**: Run `dotnet test` with 80% coverage check.

## Anti-Patterns

- **Async Void**: Using `async void` except for top-level event handlers.
- **Blocking on Async**: Using `.Result` or `.Wait()` (causes deadlocks).
- **Fat Controllers**: Putting business logic in API controllers instead of Services or MediatR handlers.
