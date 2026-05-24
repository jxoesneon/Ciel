---
name: security-scan
version: 1.0.0
description: Scan your Claude Code configuration (.claude/ directory) for security vulnerabilities, misconfigurations, and injection risks using AgentShield. Checks CLAUDE.md, settings.json, MCP servers, hooks, and agent definitions.
format: skill/1.0
author: ECC Project
license: Apache-2.0
runtimes:
  - claude-code
  - gemini-cli
  - windsurf
  - generic
triggers:
  - "security-scan"
entrypoint: SKILL.md
---

# security-scan

Scan your Claude Code configuration (.claude/ directory) for security vulnerabilities, misconfigurations, and injection risks using AgentShield. Checks CLAUDE.md, settings.json, MCP servers, hooks, and agent definitions.

## Integration

This skill is part of Ciel's core capability ecosystem.

## Origin

- **Source**: ECC (Everything Claude Code) ecosystem
- **Original Path**: /Users/meilynlopezcubero/.agents/skills/security-scan/
- **Integrated**: 2026-04-20T18:39:35-06:00
- **Council Review**: Completed per SKILL_INTEGRATION scope
