---
name: api-backend-development
version: 1.0.0
format: skill/1.0
description: CIEL's standards for REST API design, backend architecture, and server-side patterns.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(design|build|architect).*(api|backend|endpoint|rest)"
    confidence: 0.95
  - pattern: "backend patterns"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: API & Backend Development (Server-Side)

This skill formalizes the development of scalable, maintainable server-side systems. it enforces resource-based REST design and a strict layered architecture.

## API Design Mandates
1. **Resource Naming**: Use plural nouns, lowercase, kebab-case (e.g., `/api/v1/team-members`).
2. **Semantic Status Codes**:
   - `201 Created`: For successful POSTs.
   - `422 Unprocessable Entity`: For semantically invalid data (Zod/Pydantic errors).
   - `500`: Never leak details (stack traces/SQL) to the client.
3. **Pagination**: Default to Cursor-Based for large datasets; Offset-Based for admin/search.

## Layered Architecture (The 3-Layer Pattern)
- **Controller/Handler**: Request validation and response mapping. No business logic.
- **Service Layer**: Pure business orchestration. Coordinates between repositories and external APIs.
- **Repository**: Abstract data access. Maps DB rows to Domain Models.

## Performance & Optimization
- **N+1 Prevention**: Always batch fetch relationships using `IN` queries or Joins.
- **Select Specifics**: Never `SELECT *`. Only retrieve required columns.
- **Fail Gracefully**: Implement exponential backoff for all external API calls.

## Anti-Patterns
- **Verb-in-URL**: Using `/api/getUsers` instead of `GET /api/users`.
- **Impossible States**: Using boolean flag soup instead of a single `status` enum.
- **Direct DB Leaks**: Returning raw ORM/DB objects directly in the API response.
