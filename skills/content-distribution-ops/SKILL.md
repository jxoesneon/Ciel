---
name: content-distribution-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for social graph optimization, content creation, and cross-platform distribution.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(post|tweet|distribute|optimize).*(social|x|linkedin|threads|bluesky)"

    confidence: 0.9

  - pattern: "content engine"

    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Content Distribution (The Social Layer)

This skill formalizes the lifecycle of public-facing content, from network optimization to platform-native distribution. it prioritizes voice integrity and signal-to-noise ratio.

## Network Optimization (Connections)

1. **Reciprocity**: Prioritize mutual follows over one-way connections.
2. **Prune Protocol**: Surface inactive or "low-signal" accounts for review before unfollowing.
3. **Warm Paths**: Use `lead-intelligence` to identify high-bridge targets aligned with current goals.

## Content Engineering

- **Atomic Claims**: One post = one concrete claim or proof point. No engagement bait.
- **Source-First**: Derive content from real artifacts (code, memos, docs) via `brand-voice`.
- **Hard Bans**: Prohibit "Excited to share," "Game-changer," and generic "founder journey" filler.

## Cross-Platform Adaptation

- **X**: Compressed, threaded, leading with the sharpest artifact.
- **LinkedIn**: Contextual expansion for non-niche audiences; professional but non-corporate tone.
- **Threads/Bluesky**: Direct and readable; avoid feed-gaming hashtags.

## Operational Standards

- **Approval Gate**: ALL drafts must be presented to the user before publishing.
- **Voice Fingerprint**: Reuse the session's `VOICE PROFILE` for all adapted variants.

## Anti-Patterns

- **Verbatim Crosspost**: Posting the exact same copy to X and LinkedIn (erodes platform authority).
- **Fake Reflection**: Adding a moral or "lesson learned" that wasn't in the source material.
- **Amnesiac Outreach**: Drafting a DM without referencing the existing connection context.
