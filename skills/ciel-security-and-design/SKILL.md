---
name: ciel-security-and-design
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Solidity security and visual design systems.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(audit|design|style).*(solidity|amm|design system|token|css)"
    confidence: 0.9
  - pattern: "design system"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Security & Design (The Integrity Layer)

This skill manages the structural integrity of both value (Solidity) and appearance (Design Systems).

## Solidity Security (DeFi)
1. **CEI Mandate**: Checks-Effects-Interactions. Always update state BEFORE external calls.
2. **Reentrancy**: Use `nonReentrant` guards on all entry points handling token transfers.
3. **Share Math**: NEVER use `balanceOf(address(this))` directly for shares. track internal `_totalAssets`.
4. **Oracle Safety**: Spot prices are manipulable. Use TWAP (Time-Weighted Average Price).

## Design Systems
- **Tokens First**: Define primitive tokens (Colors, Spacing, Typography) in JSON/CSS vars early.
- **Hierarchy**: Enforce a clear `h1` -> `h2` -> `body` hierarchy. Prohibit random hex values.
- **Consistency Audit**: Similar elements (Buttons, Modals) MUST look identical across all pages.
- **AI Slop Detection**: Flag and remove purple-to-blue gradients, purposeless "glass morphism," and excessive rounded corners.

## Visual Accessibility
- **Contrast**: Target WCAG AA (4.5:1) for all text.
- **States**: Every interactive element MUST have distinct Hover, Focus, and Active states.

## Anti-Patterns
- **Naked CSS**: Adding styles directly to components instead of using design tokens.
- **naive mulDiv**: Performing `a * b / c` in Solidity without checking for overflow.
- **The Gradient Trap**: Using AI-suggested defaults instead of a cohesive brand palette.
