---
name: jvm-server-frameworks
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Ktor servers, Exposed ORM, and database lifecycle management.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(build|setup).*(ktor|exposed|hikari|flyway)"

    confidence: 0.9

  - pattern: "ktor patterns"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: JVM Server Frameworks (Ktor & Exposed)

This skill formalizes the use of lightweight, DSL-driven frameworks on the JVM. It focuses on the Ktor + Exposed + Flyway stack.

## Ktor Server Standards

1. **Plugin Discipline**: Install `ContentNegotiation`, `Authentication`, and `StatusPages` in the central module.
2. **Routing DSL**: Group routes by resource (`/users`, `/orders`). Extract logic to Services injected via **Koin**.
3. **Validation**: Use `require()` blocks at the route boundary for fail-fast input checking.
4. **Serialization**: Standardize on `kotlinx.serialization` with custom serializers for `Instant`.

## Exposed ORM (Persistence)

- **Transaction Safety**: All DB operations MUST run inside `newSuspendedTransaction`.
- **DSL vs DAO**:
  - Use **DSL** style for complex joins and aggregations.
  - Use **DAO** style for entity lifecycle and simple CRUD.
- **JSONB**: Use the `jsonb` column pattern with `kotlinx.serialization` for unstructured metadata.

## Database Lifecycle

- **Migrations**: Mandate **Flyway** for all schema changes. Versioned SQL scripts only.
- **Connection Pooling**: Use **HikariCP** with `TRANSACTION_READ_COMMITTED` isolation.

## Anti-Patterns

- **Implicit Nulls**: Allowing `explicitNulls = true` in serialization (leads to large, sparse JSON).
- **Blocking Transactions**: Running long-running I/O or network calls inside a `transaction` block.
- **Manual Table Creation**: Using `SchemaUtils.create` in production instead of Flyway.
