---
name: codebase-onboarding
version: 1.0.0
format: skill/1.0
description: Systematically maps an unfamiliar codebase and generates a starter CLAUDE.md to guide future agents.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(onboard|walk through|explain|understand).*(codebase|repo|project)"
    confidence: 0.9
  - pattern: "generate (claude.md|starter config)"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Codebase Onboarding (Reconnaissance)

This skill is CIEL's entry-point for any new repository. It avoids "reading everything" by using high-signal reconnaissance to build a functional map of the project.

## Workflow Phases

### Phase 1: Reconnaissance (Parallel)
- **Manifest Detection**: Identify the tech stack via `package.json`, `go.mod`, `Cargo.toml`, etc.
- **Framework Fingerprinting**: Detect Next.js, Django, FastAPI via config patterns.
- **Entry Point ID**: Find `main.*`, `index.*`, `server.*`.
- **Structural Snapshot**: Map the top 2 levels of the directory tree.

### Phase 2: Architecture Mapping
Define the:
- **Data Flow**: Trace a single request from entry (router) to exit (database).
- **Key Directories**: Map directories to their semantic purpose (e.g., `src/lib` -> Shared Utilities).

### Phase 3: Convention Detection
Identify existing patterns in:
- **Naming**: camelCase vs snake_case.
- **Error Handling**: try/catch vs Result types.
- **Testing**: Jest vs Pytest, location of test files.

### Phase 4: Artifact Generation
1. **Onboarding Guide**: A scannable 2-minute summary of the project.
2. **Starter CLAUDE.md**: Project-specific instructions and commands for CIEL sub-agents.

## Anti-Patterns
- **Context Flooding**: Reading the entire `src/` directory instead of using `glob` and `grep` for reconnaissance.
- **README Redundancy**: Copying the project description instead of adding structural and architectural insight.
- **Instruction Over-Engineering**: Generating a `CLAUDE.md` longer than 100 lines.
