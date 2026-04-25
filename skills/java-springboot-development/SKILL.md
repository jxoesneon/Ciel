---
name: java-springboot-development
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Java 17+ and Spring Boot 3+ development, security, and TDD.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|review).*(java|spring boot|springboot)"
    confidence: 0.95
  - pattern: "spring security"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Java & Spring Boot (The Enterprise Layer)

This skill formalizes the development of production-grade Java services using Spring Boot. It mandates immutability, fail-fast error handling, and security-first configuration.

## Java Standards (17+)
1. **Immutability**: Favor `record` and `final` fields. No setters.
2. **Null Safety**: Accept `@Nullable` only when unavoidable. Use `Optional` for return types.
3. **Fail Fast**: Use domain-specific unchecked exceptions (e.g., `EntityNotFoundException`).
4. **Streams**: Keep pipelines short and readable. Favor `List.of()` and `.toList()`.

## Spring Boot Architecture
- **Layered Discipline**: Controller (thin) -> Service (pure logic) -> Repository (abstract access).
- **Injection**: Use constructor injection; prohibit `@Autowired` on fields.
- **Transactions**: Use `@Transactional(readOnly = true)` for all GETs.
- **Validation**: Enforce `@Valid` on all RequestBody DTOs via Bean Validation (Zod-like patterns).

## Security Review
- **Auth**: Default to stateless JWT. Use `OncePerRequestFilter` for validation.
- **CSRF**: Disable for pure APIs; Enable for session-based browser apps.
- **SQLi**: Prohibit string concatenation in `@Query`. Use `:param` bindings only.
- **PII**: Never log full payloads or sensitive fields. Redact centrally.

## TDD Workflow
- **Unit**: JUnit 5 + AssertJ + Mockito. 100% logic coverage.
- **Web**: `MockMvc` for controller boundary testing.
- **Persistence**: `DataJpaTest` with **Testcontainers** (no H2 in CI).

## Anti-Patterns
- **Lombok Over-use**: Using `@Data` on immutable domain entities (use `@Value` or `record`).
- **Magic Numbers**: Hardcoding timeouts or page sizes.
- **Silent Catch**: Catching `Exception` without rethrowing or structured logging.
