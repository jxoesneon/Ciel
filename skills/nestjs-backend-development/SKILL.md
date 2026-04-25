---
name: nestjs-backend-development
version: 1.0.0
format: skill/1.0
description: Modular NestJS architecture, DTO validation, Guards, and Type-Safe backend patterns.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|review).*(nest.js|nestjs|decorator|provider)"
    confidence: 0.95
  - pattern: "nest generate"
    confidence: 1.0
---

# CIEL ADAPTATION: NestJS Development (Modular TypeScript)

This skill formalizes the development of modular, type-safe backends using NestJS. it prioritizes dependency injection and declarative validation.

## Modular Discipline
1. **Feature Isolation**: One module per domain (e.g., `UsersModule`, `AuthModule`).
2. **Thin Controllers**: Controllers MUST only handle HTTP parsing, calling a Service, and returning a DTO.
3. **Pure Services**: Put all business logic, external API calls, and DB orchestration in injectable Services.
4. **Common Layer**: Put cross-cutting concerns (Interceptors, Filters, Decorators) in `src/common/`.

## Data Integrity (The DTO Pattern)
- **Validation**: Mandate `class-validator` decorators on all Request DTOs.
- **Hard Gate**: Enable `whitelist: true` and `forbidNonWhitelisted: true` in the global `ValidationPipe`.
- **Serialization**: Use `ClassSerializerInterceptor` to exclude internal fields (e.g., `passwordHash`) from responses.

## Security & Auth
- **Guards**: Use `JwtAuthGuard` for session-validation. Do NOT perform resource-specific auth in guards (use Services).
- **Environment**: Use `@nestjs/config` with a validation schema (Joi/Zod) to ensure all required env vars are present at boot.

## Anti-Patterns
- **Direct ORM Leakage**: Returning raw Prisma/TypeORM entities directly in the controller (leaks DB schema).
- **Circular Modules**: Allowing `Module A` to import `Module B` while `Module B` imports `Module A` (use `forwardRef`).
- **Global Everything**: Exporting every service from every module. Export ONLY what is genuinely shared.
