---
name: container-and-deployment
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Docker containerization and CI/CD deployment strategies.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(deploy|docker|ci/cd|k8s).*(container|rolling|canary|github action)"

    confidence: 0.9

  - pattern: "deployment patterns"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Containers & Deployment (The Ship Layer)

This skill formalizes the packaging and delivery of services. it prioritizes reproducible builds and zero-downtime rollouts.

## Docker Patterns

1. **Multi-Stage Mandate**: Separate `deps` -> `builder` -> `runner`. Runner MUST be minimal (e.g., `alpine`) and non-root.
2. **Layer Optimization**: Copy dependency files (`package.json`, `go.mod`) first to maximize cache hits.
3. **Health Checks**: ALWAYS include a `HEALTHCHECK` instruction using `wget` or a simple script.
4. **Security**: Prohibit `:latest` tags. Use pinned versions (e.g., `node:22.12-alpine`).

## Deployment Strategies

- **Rolling**: Default for non-breaking changes. Requires backward-compatible DB schemas.
- **Blue-Green**: Mandate for critical services or major breaking changes. Requires 2x infra capacity.
- **Canary**: Use for high-risk UI or performance changes. Start with 5% traffic.

## CI/CD Pipeline

- **Gate 1: Quality**: Lint -> Typecheck -> Unit Tests.
- **Gate 2: Security**: Dependency Audit -> Secret Grep.
- **Gate 3: Performance**: Benchmark vs Baseline.
- **Deploy**: Environment-specific secrets injected at runtime; never baked into images.

## Anti-Patterns

- **The Giant Image**: Including `node_modules` or build tools in the final production image.
- **Root Runner**: Running the container process as `UID 0`.
- **Blind Migrations**: Running destructive DB migrations during a rolling deployment.
