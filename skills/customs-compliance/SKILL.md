---
name: customs-compliance
version: 1.0.0
format: skill/1.0
description: Tariff classification, documentation standards, and regulatory screening for international trade.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(classify|export|import|customs).*(hs code|tariff|compliance|border)"
    confidence: 0.9
  - pattern: "denied party screening"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Customs Compliance (Trade Integrity)

This skill formalizes the "Border Layer" of CIEL. It ensures lawful movement of goods across jurisdictions while optimizing duty exposure.

## Tariff Classification (GRI Hierarchy)
The Orchestrator MUST apply GRI rules in strict order:
- **GRI 1**: Heading text and Section/Chapter notes (resolves ~90%).
- **GRI 3(a)**: Specificity (prefer "Surgical Gloves" over "Rubber Articles").
- **GRI 3(b)**: Essential Character (for composite goods and sets).

## Compliance Mandates
1. **Restricted Party Screening**: Screen Buyer, Seller, Consignee, and Bank against SDN (OFAC) and Entity (BIS) lists.
2. **Prior Disclosure**: If a violation is discovered, file a prior disclosure before government investigation begins to cap penalties.
3. **ISF 10+2 (US)**: File 24 hours before vessel loading.

## Documentation Standard
- **Commercial Invoice**: Must include description, unit price, Incoterms 2020, and Country of Origin.
- **Valuation**: Default to Transaction Value (Method 1). Hierarchically fall back to Deductive or Computed methods if needed.

## Anti-Patterns
- **Vibes-Classification**: Classifying from a product name alone without technical specs.
- **Incoterm Blur**: Using EXW for international trade (creates export compliance burdens for the buyer).
- **Amnesiac Adjudication**: Clearing a screening hit without documenting the rationale (date, tool, reason).
