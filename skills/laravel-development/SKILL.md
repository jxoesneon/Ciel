---
name: laravel-development
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Laravel development, security, TDD, and CI verification.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|review).*(laravel|php|artisan)"
    confidence: 0.9
  - pattern: "laravel patterns"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Laravel Development (The Modern PHP Layer)

This skill formalizes the development of high-quality Laravel applications. It mandates thin controllers, action-based logic, and a strict "RefreshDatabase" testing strategy.

## Architecture & Patterns
1. **Layered Boundaries**: Controller (thin) -> Actions/Services (use cases) -> Models (pure domain).
2. **Routing**: Use Route-Model binding and scoped bindings to prevent cross-tenant access.
3. **Data Access**: Enforce Eager Loading (`with()`) to prevent N+1 queries. Use `snake_case` and plural table names.
4. **IO Efficiency**: Move all heavy work (Email, PDF, Integrations) to Queued Jobs.

## Security Standards
- **Mass Assignment**: ALWAYS use `$fillable` or `$guarded`. Prohibit `Model::unguard()`.
- **Validation**: Mandate `FormRequest` classes for all controller inputs.
- **XSS**: Default to Blade `{{ }}` escaping. Use `{!! !!}` ONLY for sanitized, trusted HTML.
- **CSRF**: Maintain `VerifyCsrfToken` for all session-based browser routes.

## TDD & Verification
- **Framework**: Prefer **Pest** for new tests. Use `RefreshDatabase` for clean state.
- **Mocks**: Use `Bus::fake()`, `Queue::fake()`, and `Mail::fake()` for side effects.
- **CI Loop**: Clean build -> `pint --test` -> `phpstan analyse` -> `artisan test --coverage` (80% gate).

## Anti-Patterns
- **Fat Models**: Putting complex business logic in Eloquent models instead of Actions.
- **Blind Merging**: Merging dependencies without running `composer audit`.
- **Impossible States**: Using boolean flags in the DB instead of a `status` enum or state machine.
