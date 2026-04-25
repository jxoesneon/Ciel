---
name: workflow-and-platform-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Git workflows, GitHub operations, and on-device LLM patterns.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(manage|branch|release|triage).*(git|github|ios|foundationmodel)"

    confidence: 0.9

  - pattern: "git flow"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Workflow & Platform (The Delivery Layer)

This skill manages the standard mechanisms of collaboration and specialized platform features (iOS).

## Git & GitHub Ops

1. **GitHub Flow**: Default to short-lived feature branches and PRs. Main MUST always be deployable.
2. **Commits**: Follow Conventional Commits (`feat(scope): ...`). body must explain WHY, not WHAT.
3. **Triage**: Apply Type (bug, feature) and Priority (critical, high) labels to all issues.
4. **Security**: Audit Dependabot and secret scanning alerts weekly. No un-investigated CI failures.

## iOS On-Device LLM (FoundationModels)

- **Availability**: ALWAYS check `SystemLanguageModel.default.availability` before session creation.
- **Context Window**: Combination of Instructions + Prompt + Output MUST NOT exceed 4,096 tokens.
- **Structured Output**: Use `@Generable` types for extraction instead of raw string parsing.
- **Privacy**: Rely on on-device execution for PHI/PII tasks. Data MUST NOT leave the device.

## Operational Discipline

- **Linear History**: Favor Rebase for local updates; Merge for PR integration to preserve exact context.
- **Changelogs**: Generate from Conventional Commits. Separate Features, Fixes, and Breaking Changes.

## Anti-Patterns

- **Amnesiac PRs**: Merging a PR with failing CI checks or outstanding review comments.
- **Token Overload**: Sending > 4k tokens to an on-device iOS model (causes truncation/failure).
- **Naked New**: Calling `new` in C++ or creating un-managed sessions in iOS without cleanup.
