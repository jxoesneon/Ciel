---
name: e2e-and-visual-verification
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Playwright E2E testing and intentional frontend design.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(test|design|build).*(e2e|playwright|frontend|ui|visual)"
    confidence: 0.95
  - pattern: "page object model"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: E2E & Visual Verification (The Interface Layer)

This skill manages the correctness and quality of user interfaces. it prioritizes stable test patterns and intentional design choices over generic UI.

## E2E Testing (Playwright)
1. **POM Mandate**: Use Page Object Models for all tests to isolate selectors from logic.
2. **Stability**: Prohibit `waitForTimeout()`. Use auto-waiting locators or `waitForResponse()`.
3. **Flakiness Protocol**: Quarantine flaky tests with `test.fixme()`. Identify root causes via `--repeat-each=10`.
4. **Artifacts**: ALWAYS capture screenshots and traces on failure in CI.

## Intentional Frontend Design
- **Direction**: Pick a visual direction (Minimal, Industrial, Editorial) and commit to it.
- **Hierarchy**: Use strong whitespace and asymmetrical overlaps to clarify focus.
- **Motion**: Use animation only to reveal hierarchy or reinforce action. One well-directed load sequence > many micro-interactions.
- **Tokens**: Use CSS variables for all colors, spacing, and typography. No random hex values.

## Visual Regression
- **Breakpoints**: Verify layouts at Mobile (375px), Tablet (768px), and Desktop (1440px).
- **Hard Gate**: Flag layout shifts > 5px or overlapping elements in PR reviews.

## Anti-Patterns
- **Generic AI UI**: Defaulting to purple-gradient-on-white card grids without a specific brand reason.
- **Selector Fragility**: Using brittle CSS selectors in E2E tests instead of `data-testid`.
- **Amnesiac Cleanup**: Creating E2E data (e.g., users) without a teardown or `RefreshDatabase` strategy.
