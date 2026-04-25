---
name: ciel-documentation-engine
version: 1.0.0
format: skill/1.0
description: CIEL's framework for high-integrity documentation generation, validation, and semantic indexing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "documentation", "domain:systems"]
triggers:
  - pattern: '(generate|update|validate).*(docs|documentation|readme)'
    confidence: 1.0
  - pattern: "semantic indexing"
    confidence: 0.9
---

# CIEL ADAPTATION: Documentation Engine

This framework governs the lifecycle of technical documentation within the CIEL ecosystem. It ensures that all documentation is accurate, spec-compliant, and semantically queryable.

## Core Capabilities
1. **Spec-Doc Synchronization**: Automatically validates that documentation (e.g., `README.md`) stays in sync with actual project structure and metrics.
2. **Semantic Indexing**: Prepares documentation for high-density RAG (Retrieval-Augmented Generation) by enforcing structured headings and metadata.
3. **Format Enforcement**: Validates and auto-fixes Markdown standards (MD040, MD060) via integrated linting hooks.
4. **Template Application**: Provides standardized templates for new skills, subagents, and council reports.

## Mandates
- **Single Source of Truth**: Metrics (like skill counts) MUST be derived from implementation, not hand-coded.
- **Link Integrity**: Every internal link in the documentation must be verified for resolution.
- **Tone & Style**: Documentation must maintain a professional, senior-engineering tone.

## Automation Hooks
- **Post-Build**: Generates updated `MANIFEST.md` and `INDEXING.md`.
- **Pre-Commit**: Runs `markdownlint` and verifies version consistency.

## Anti-Patterns
- **Stale Metrics**: Quoting version numbers or counts that do not match the current repo state.
- **Broken References**: Linking to deleted files or private directories in public documentation.
