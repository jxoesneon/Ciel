---
name: opensource-and-repo-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for public release pipelines and cross-stack repo scanning.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:

  - pattern: "(open source|sanitize|scan|audit).*(repo|project|github|secret)"

    confidence: 0.9

  - pattern: "repo scan html report"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Open-Source & Repo Ops (The Public Layer)

This skill formalizes the "Private-to-Public" transition and structural auditing of massive codebases.

## The Open-Source Pipeline

1. **Fork**: Strip `.git`, `node_modules`, and local configs. Replace internal references with placeholders.
2. **Sanitize (Hard Gate)**: Scan for secrets (sk-), PII, and internal server names. PROHIBIT release on FAIL.
3. **Package**: Generate `CLAUDE.md`, `.env.example`, `LICENSE`, and `setup.sh`.
4. **Verify**: Run a final scan on the staging directory before publishing to GitHub.

## Cross-Stack Repo Scanning

- **File Classification**: Tag files as Project Code, Third-Party (Vendored), or Build Artifacts.
- **Library Detection**: Identify embedded libraries (e.g., FFmpeg, OpenSSL) and their vintage.
- **Verdicts**:
  - **Core Asset**: Keep/Optimize.
  - **Extract & Merge**: Move to dedicated package managers.
  - **Deprecate**: Remove unused/dead code.
- **HTML Reports**: Produce interactive drill-down reports for monorepo reorganization.

## Operational Safety

- **Approval**: NEVER push a repo to public GitHub without a final manual review of the `SANITIZATION_REPORT.md`.
- **Reproducibility**: Commit repo-scan results and sanitization configs to the internal audit trail.

## Anti-Patterns

- **Blind Push**: Pushing a sanitized repo without re-running tests in the new clean environment.
- **Vibe-Cleanup**: Deleting files based on name instead of cross-stack scanning results.
- **Internal-Key Leak**: Forgetting to update `.env.example` with generic keys, leaving private URLs intact.
