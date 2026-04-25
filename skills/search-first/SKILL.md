---
name: search-first
version: 1.0.0
format: skill/1.0
description: CIEL's mandated 'Research-Before-Strategy' workflow. Enforces the discovery of existing solutions, MCPs, and libraries before writing net-new code.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(add|build|implement|create).*(feature|functionality|integration|utility)"
    confidence: 0.9
  - pattern: "how should (we|I) solve"
    confidence: 0.85
  - pattern: "write a (script|wrapper|client) for"
    confidence: 0.8

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Search-First (Research Mandate)

This skill operationalizes CIEL's core **"Research-Before-Strategy"** mandate. It acts as a hard gate preventing the premature generation of net-new code when robust, maintained solutions already exist in the ecosystem, the repository, or via MCP servers.

## Integration Context

Adapted from `~/.agents/skills/search-first/`. This skill heavily influences the **Strategy Layer** (specifically the `make-plan` skill). It dictates that the initial phase of any non-trivial implementation MUST be a dedicated research turn.

## The Search-First Decision Matrix

Before proposing a custom implementation, agents must evaluate existing solutions against this matrix:

| Signal / Finding | Required Action |
| :--- | :--- |
| Exact match, well-maintained, MIT/Apache | **Adopt**: Propose installation. **REQUIRE HUMAN CONFIRMATION** before modifying package files. Do not wrap unnecessarily. |
| Partial match, strong foundation | **Extend**: Propose installation. **REQUIRE HUMAN CONFIRMATION**. Write a thin, specific wrapper. |
| Multiple weak matches | **Compose**: Combine small packages. **REQUIRE HUMAN CONFIRMATION** for all new dependencies. |
| Nothing suitable found | **Build**: Write custom code, informed by the research phase. |

## Orchestration Workflow

When triggered, execute the following workflow:

1. **Need Analysis**: Define the exact constraints (language, framework, license).
2. **Local Context Search**: 
   - Use `grep_search` and `glob` to verify the functionality doesn't already exist in the repo.
   - Check available MCP servers (e.g., via `cli_help` or direct MCP tool listing).
3. **Ecosystem Search**: 
   - If internet tools (like `google_web_search` or `exa-search`) are available, search the relevant package manager (npm, PyPI, crates.io).
   - Evaluate community trust, download counts, and recent update activity to mitigate supply chain risks.
4. **Evaluate & Decide**: Score candidates and apply the Decision Matrix.
5. **Documentation (Evolution)**: If a new external dependency is adopted, the agent MUST document the rationale, version, and alternatives considered in the project's architectural log (e.g., `ARCHITECTURE.md`) or via memory.

## Anti-Patterns (Enforced by Auditor)

- **The "Not Invented Here" Syndrome**: Writing custom HTTP clients, complex validation logic, or standard algorithms without checking the ecosystem.
- **Ignoring MCPs**: Hardcoding API integrations when an MCP server is available in the workspace.
- **Dependency Bloat**: Pulling in a massive, multi-megabyte framework just to use a single string-manipulation function (e.g., pulling all of `lodash` for a simple `camelCase` conversion).

## Council Alignment
This skill is critical for **Ciel-Efficiency** (preventing architectural bloat) and **Ciel-Safety** (preferring battle-tested, community-audited code over unverified custom implementations).
