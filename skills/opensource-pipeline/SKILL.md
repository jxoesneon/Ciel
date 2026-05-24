---
name: opensource-pipeline
version: 1.0.0
description: Open-source pipeline: fork, sanitize, and package private projects for safe public release. Chains 3 agents (forker, sanitizer, packager). Triggers: /opensource, open source this, make this public, prepare for open source.
format: skill/1.0
author: ECC Project
license: Apache-2.0
runtimes:
  - claude-code
  - gemini-cli
  - windsurf
  - generic
triggers:
  - "opensource-pipeline"
entrypoint: SKILL.md
---

# opensource-pipeline

Open-source pipeline: fork, sanitize, and package private projects for safe public release. Chains 3 agents (forker, sanitizer, packager). Triggers: /opensource, open source this, make this public, prepare for open source.

## Integration

This skill is part of Ciel's core capability ecosystem.

## Origin

- **Source**: ECC (Everything Claude Code) ecosystem
- **Original Path**: /Users/meilynlopezcubero/.agents/skills/opensource-pipeline/
- **Integrated**: 2026-04-20T18:44:26-06:00
- **Council Review**: Completed per SKILL_INTEGRATION scope
