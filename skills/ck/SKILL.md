---
name: ck
version: 1.0.0
description: Persistent per-project memory for Claude Code. Auto-loads project context on session start, tracks sessions with git activity, and writes to native memory. Commands run deterministic Node.js scripts — behavior is consistent across model versions.
format: skill/1.0
author: ECC Project
license: Apache-2.0
runtimes:
  - claude-code
  - gemini-cli
  - windsurf
  - generic
triggers:
  - "ck"
entrypoint: SKILL.md
---

# ck

Persistent per-project memory for Claude Code. Auto-loads project context on session start, tracks sessions with git activity, and writes to native memory. Commands run deterministic Node.js scripts — behavior is consistent across model versions.

## Integration

This skill is part of Ciel's core capability ecosystem.

## Origin

- **Source**: ECC (Everything Claude Code) ecosystem
- **Original Path**: /Users/meilynlopezcubero/.agents/skills/ck/
- **Integrated**: 2026-04-20T18:43:32-06:00
- **Council Review**: Completed per SKILL_INTEGRATION scope
