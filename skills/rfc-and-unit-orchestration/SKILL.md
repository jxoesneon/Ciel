---
name: rfc-and-unit-orchestration
version: 1.0.0
format: skill/1.0
description: CIEL's framework for RFC-driven feature implementation and unit-based orchestration.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(rfc|decompose|unit).*(feature|dag|work unit|merge queue)"
    confidence: 0.95
  - pattern: "ralphinho rfc pipeline"
    confidence: 1.0
---

# CIEL ADAPTATION: RFC & Unit Orchestration (The Structure Layer)

This skill formalizes the decomposition of large features into independently verifiable work units. it prioritizes quality gates and merge safety.

## The RFC Pipeline
1. **Intake**: Map high-level intent to a formal Request for Comments (RFC).
2. **DAG Decomposition**: Split the RFC into a Directed Acyclic Graph (DAG) of work units.
3. **Unit Assignment**: Assign units based on complexity tiers (1: Isolated, 2: Multi-file, 3: Schema/Security).
4. **Merge Queue**: Manage the sequential integration of units. Re-run integration tests after EVERY merge.

## Work Unit Specification
Every unit MUST include:
- `id` and `depends_on`.
- `scope`: Exact files or functions.
- `acceptance_tests`: Minimum proving steps.
- `rollback_plan`: How to undo the change if integration fails.

## Implementation Workflow
- **Research**: iterative context retrieval for the unit.
- **Implementation**: TDD-based implementation.
- **Review**: Mandatory code-review turn before merge-readiness.
- **Integration**: Rebase and resolve conflicts on the main feature branch.

## Anti-Patterns
- **Zombie Dependencies**: Proceeding with a unit when its required predecessor failed.
- **Merge-and-Forget**: Integrating a unit without re-running the full integration test suite.
- **The Giant Unit**: Defining a single unit that touches > 5 files across different domains.
