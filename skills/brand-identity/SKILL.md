---
name: brand-identity
version: 1.0.0
format: skill/1.0
description: CIEL's framework for brand voice, visual identity standards, and asset management.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(define|update|audit).*(brand|voice|identity|tone)"
    confidence: 0.9
  - pattern: "brand guidelines"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Brand Identity (Voice & Vision)

This skill formalizes the "Brand Layer" of CIEL. It ensures consistency across all user-facing content, from terminal output to marketing assets.

## The Brand Core
1. **Voice Framework**: Define Tone (Professional, Playful, Stoic) and Style (Concise, Narrative, technical).
2. **Visual Identity**: Standards for colors (Hex/Tokens), Typography (Google Fonts/System), and Logo usage.
3. **Messaging**: Canonical value propositions and "Hero" statements.

## Operational Workflow
### 1. Brand Context Injection
The Orchestrator MUST extract brand context before drafting user-facing content:
- Read `docs/brand-guidelines.md` (Source of Truth).
- Map findings to `assets/design-tokens.json` for technical consistency.

### 2. Asset Validation
- **Naming**: kebab-case for all media assets.
- **Format**: SVGs for icons, WebP/AVIF for images.
- **Audit**: Compare new assets against the existing color palette to ensure zero drift.

## Anti-Patterns
- **Voice Drift**: Mixing "Bro-talk" with "Corporate-speak" in the same session.
- **Raw Hex Abuse**: Hardcoding colors instead of using the synced token system.
- **Orphaned Assets**: Storing brand images in root instead of `assets/brand/`.
