---
name: product-lens
version: 1.0.0
format: skill/1.0
description: A diagnostic framework for validating product direction and feature viability before engineering begins.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(analyze|diagnose|review).*(product|idea|founder lens)"

    confidence: 0.9

  - pattern: "should I build (this|that)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Product Lens (Discovery & Viability)

This skill formalizes the "Discovery" phase of CIEL development. It asks the hard questions required to validate a feature's "Why" before a single line of code is planned.

## Diagnostic Modes

### 1. The Core 7 Diagnostic

The Orchestrator MUST answer these 7 questions before authorizing a new project:

- **Who**: Specific user persona.
- **Pain**: Quantifiable problem being solved.
- **Why Now**: What technical or market shift makes this possible?
- **The 10-Star**: The "Unlimited Budget" version of the feature.
- **The MVP**: Smallest version that proves the core thesis.
- **Anti-Goal**: What this feature explicitly will NOT do.
- **Success Metric**: How we will know it works (not vibes).

### 2. User Journey Audit

For existing features, the Orchestrator MUST:

- Clone/Install as a fresh user.
- Document every friction point and missing doc.
- Time the "Time-to-Value" (TTV).
- Recommend the top 3 friction-reduction fixes.

### 3. Feature Prioritization (ICE Score)

When choosing between multiple ideas, rank them using:

- **Impact** (1-5)
- **Confidence** (1-5)
- **Ease** (1-5)
- **Result**: (Impact * Confidence) / Ease.

## Output Artifact

Produces a `PRODUCT-BRIEF.md` containing the diagnosis and a clear GO/NO-GO recommendation.

## Anti-Patterns

- **Founder-Theater**: Building a feature just because it "sounds cool" without defining a success metric.
- **Invisible Friction**: Ignoring a 5-minute setup hurdle because the developer "already has it installed."
- **Scope Creep**: Expanding the MVP into the 10-star version before the thesis is proven.
