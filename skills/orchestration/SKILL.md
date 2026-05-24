---
name: orchestration
version: 1.0.0
description: Multi-agent orchestration for complex tasks. Use when tasks require parallel work, multiple agents, or sophisticated coordination. Triggers include requests for features, reviews, refactoring, testing, documentation, or any work that benefits from decomposition into parallel subtasks. This skill defines how to orchestrate work using cc-mirror tasks for persistent dependency tracking and TodoWrite for real-time session visibility.
format: skill/1.0
author: ECC Project
license: Apache-2.0
runtimes:
  - claude-code
  - gemini-cli
  - windsurf
  - generic
triggers:
  - "orchestration"
entrypoint: SKILL.md
---

# orchestration

Multi-agent orchestration for complex tasks. Use when tasks require parallel work, multiple agents, or sophisticated coordination. Triggers include requests for features, reviews, refactoring, testing, documentation, or any work that benefits from decomposition into parallel subtasks. This skill defines how to orchestrate work using cc-mirror tasks for persistent dependency tracking and TodoWrite for real-time session visibility.

## Integration

This skill is part of Ciel's core capability ecosystem.

## Origin

- **Source**: ECC (Everything Claude Code) ecosystem
- **Original Path**: /Users/meilynlopezcubero/.agents/skills/orchestration/
- **Integrated**: 2026-04-20T18:32:07-06:00
- **Council Review**: Completed per SKILL_INTEGRATION scope
