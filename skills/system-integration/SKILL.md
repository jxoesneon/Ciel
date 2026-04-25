---
name: system-integration
version: 1.0.0
format: skill/1.0
description: Decoupling business logic using Hexagonal Architecture and building repo-native API connectors.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(integrate|build|add).*(connector|provider|integration|adapter)"

    confidence: 0.9

  - pattern: "hexagonal architecture"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: System Integration (Ports & Adapters)

This skill formalizes the "Hexagonal" approach to integrations. It ensures business logic remains independent of frameworks, transport, and external APIs.

## The Hexagonal Core

1. **Domain Model**: Pure entities and business rules. Zero framework imports.
2. **Ports (Interfaces)**:
   - **Inbound**: What the app can do (e.g., `CreateOrderPort`).
   - **Outbound**: What the app needs (e.g., `PaymentGatewayPort`).
3. **Adapters (Impls)**: Concrete implementations (e.g., `StripeAdapter`, `PostgresRepo`).

## Connector Builder Workflow (The House Style)

When adding a new integration, the Orchestrator MUST:

1. **Audit Patterns**: Inspect 2+ existing connectors to map file layout and config schema.
2. **Match Boundaries**: Do not invent a new architecture; use the host repo's existing abstraction layers.
3. **Registry Wiring**: Ensure the new connector is correctly registered in the host's discovery/DI system.

## Validation Standard

- **Unit Test**: Test UseCases with memory-fakes for ports.
- **Contract Test**: Verify that adapters fulfill the port's interface requirements.
- **Evidence-First**: Include live proof of successful auth/handshake in the final report.

## Anti-Patterns

- **Adapter Leakage**: Leaking SDK-specific types (e.g., `Stripe.Charge`) into the domain layer.
- **Cargo-Culting**: Copying an old, deprecated connector pattern just because it exists.
- **Blind Transport**: Writing transport code without error-handling or retry logic.
