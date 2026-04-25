---
name: investor-relations
version: 1.0.0
format: skill/1.0
description: Managing investor outreach, communications, and fundraising materials with high credibility.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(fundraise|investor).*(outreach|email|pitch|deck|memo|model)"
    confidence: 0.95
  - pattern: "reply to (vc|investor)"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Investor Relations (Fundraising)

This skill formalizes the "Investor Layer" of CIEL. It ensures that all communications and materials are concrete, personalized, and architecturally consistent.

## The Single Source of Truth
Before drafting any material, the Orchestrator MUST confirm the "Fundraising Manifest" (stored in `docs/fundraising/MANIFEST.md`):
- **Traction**: MRR, users, growth.
- **Raise**: Amount, instrument, use of funds.
- **Story**: Why now? The wedge.

## Outreach Rules
1. **Personalization**: Reference portfolio companies or a specific partner's thesis.
2. **Hard Bans**: Delete "excited to share", "game-changer", and begging language.
3. **Low Friction**: Every outreach MUST end with a single, concrete next step (the "Ask").

## Materials Standard
- **Pitch Deck**: 12-slide standard flow (Problem/Solution/Market/Traction/Team/Ask).
- **One-Pager**: Executive summary + credible proof points early.
- **Financial Model**: Assumption-drivenRev logic with base/bull cases.

## Anti-Patterns
- **The Fan-Out**: Sending generic "spray and pray" emails to 50 investors.
- **Number Jitter**: Reporting different traction numbers in the deck vs. the email.
- **Soft Closes**: Ending an email with "Let me know what you think" instead of a meeting request.
