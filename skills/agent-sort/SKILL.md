---
name: agent-sort
version: 1.0.0
description: Build an evidence-backed ECC install plan for a specific repo by sorting skills, commands, rules, hooks, and extras into DAILY vs LIBRARY buckets using parallel repo-aware review passes. Use when ECC should be trimmed to what a project actually needs instead of loading the full bundle.
format: skill/1.0
author: ECC Project
license: Apache-2.0
runtimes:
  - claude-code
  - gemini-cli
  - windsurf
  - generic
triggers:
  - "agent-sort"
entrypoint: SKILL.md
---

# agent-sort

Build an evidence-backed ECC install plan for a specific repo by sorting skills, commands, rules, hooks, and extras into DAILY vs LIBRARY buckets using parallel repo-aware review passes. Use when ECC should be trimmed to what a project actually needs instead of loading the full bundle.

## Integration

This skill is part of Ciel's core capability ecosystem.

## Origin

- **Source**: ECC (Everything Claude Code) ecosystem
- **Original Path**: /Users/meilynlopezcubero/.agents/skills/agent-sort/
- **Integrated**: 2026-04-20T18:43:31-06:00
- **Council Review**: Completed per SKILL_INTEGRATION scope
