---
name: quality-and-returns
version: 1.0.0
format: skill/1.0
description: CIEL's framework for manufacturing quality (NCR/CAPA) and reverse logistics (Returns/Grading).
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(investigate|process).*(non-conformance|ncr|capa|return|refund)"
    confidence: 0.95
  - pattern: "root cause analysis"
    confidence: 1.0
---

# CIEL ADAPTATION: Quality & Returns (Integrity & Recovery)

This skill formalizes the lifecycle of non-conforming physical goods, from manufacturing defects to customer returns. It prioritizes safety, regulatory compliance, and margin recovery.

## Non-Conformance Lifecycle (NCR/CAPA)
1. **Identify & Contain**: Red-tag material immediately. Electronic hold in ERP.
2. **Investigate (RCA)**: Use 5-Why for simple issues; Ishikawa (6M) for complex process failures.
3. **Disposition (MRB)**: Use-as-is (requires justification), Rework, Repair, Scrap, or RTV.
4. **CAPA**: Initiate for repeat failures or safety risks. Validate effectiveness (90-day monitoring).

## Reverse Logistics (Returns)
- **Grading**:
  - **Grade A**: Like New. Restock with full margin.
  - **Grade B**: Minor cosmetic wear. Open-box/Refurb.
  - **Grade C/D**: Heavy wear/Damage. Liquidate, parts-harvest, or destroy.
- **Fraud Detection**: Flag for review if return rate > 30% or serial number mismatch (+40 points).
- **Vendor Recovery**: Pursue RTV if (Credit * Probability) > (Labor + Shipping).

## Decision Framework: "Human Error"
Human error is NEVER a root cause. The Orchestrator MUST ask: "Why did the system allow this error?" (e.g., Lack of poka-yoke, poor UI, fatigue).

## Anti-Patterns
- **Amnesiac CAPA**: Closing a CAPA at implementation without a 90-day effectiveness verification.
- **Vibes-Disposition**: "Using-as-is" because of production pressure without engineering risk assessment.
- **Invisible Returns**: Letting RTV-eligible product sit in the warehouse past the vendor claim window.
