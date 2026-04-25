---
name: ciel-project-scaffolder
version: 1.0.0
format: skill/1.0
description: CIEL's framework for standardized project initialization, module creation, and boilerplate management.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(scaffold|create|init|new).*(project|module|service|boilerplate)"
    confidence: 0.9
  - pattern: "standardize project"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Project Scaffolder (The Foundation)

This skill ensures that every new piece of CIEL code starts with high architectural integrity and standard-compliant structure.

## Scaffolding Tiers
1. **The Project**: Create the root structure (src, tests, docs, .github, .ciel).
2. **The Module**: Create a new domain or service folder with its own `README.md` and `tests/`.
3. **The Infrastructure**: Generate Dockerfiles, CI/CD YAML, and `CLAUDE.md` / `GEMINI.md` context files.

## Standardization Rules
- **Structure**: Always use the project's detected pattern (e.g., src/internal, pkg, app/).
- **Linting**: Run `npm init`, `go mod init`, or `cargo init` followed by the project's formatter.
- **Context**: Every new directory MUST contain a `.md` file explaining its purpose to future agents.

## Anti-Patterns
- **The Naked Folder**: Creating a directory without a README or context file.
- **Boilerplate Bloat**: Including "just-in-case" libraries or code that isn't requested.
- **Pattern Breaking**: Introducing a `service/` folder in a project that uses `logic/` folders.
