---
name: logistics-management
version: 1.0.0
format: skill/1.0
description: Managing freight exceptions, carrier portfolios, and rate negotiations with evidence-first resolution.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(manage|negotiate|resolve).*(carrier|freight|shipment|delay|claim)"
    confidence: 0.95
  - pattern: "freight exceptions"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Logistics Management (Carrier & Claims)

This skill formalizes the "Transportation Layer." It ensures Quick resolution of shipment exceptions and strategic management of carrier relationships.

## Exception Resolution (OS&D)
1. **Classify**: Delay, Damage (Visible/Concealed), Shortage, or Refusal.
2. **Document**: Capture POD exceptions, photos, and commercial invoices.
3. **Trace**: Trigger loss protocol at 24h past ETA for FTL; 48h for LTL.
4. **Claim**: File within the 9-month Carmack window (US) or 14-day Montreal window (Air).

## Carrier Portfolio Strategy
- **Routing Guide**: Maintain a 3-deep guide (Primary, Secondary, Tertiary/Broker).
- **Rate Components**: Negotiate Linehaul, Fuel Surcharge (FSC), and Accessorials (Detention/Liftgate) independently.
- **Scorecarding**: Track OTD (Target >= 95%) and Tender Acceptance (Target >= 90%).

## Communication Tone
- **Neutral/Collaborative**: For routine exceptions and good partners.
- **Formal/Fact-Based**: For major exceptions, patterns, or denied claims.
- **Proactive/Empathetic**: For customer-facing delay notifications.

## Anti-Patterns
- **Blind Acceptance**: Signing a clean POD without inspecting the count/condition.
- **Bundled Rates**: Accepting a "flat" rate that hides excessive fuel or detention costs.
- **Silent Redundancy**: Keeping 10+ carriers on a low-volume lane (dilutes leverage).
