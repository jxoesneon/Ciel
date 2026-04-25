---
name: billing-and-burn-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for revenue truth and token-burn auditing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(audit|analyze|remediate).*(billing|burn|cost|revenue|stripe)"
    confidence: 0.95
  - pattern: "ecc tools cost audit"
    confidence: 1.0
---

# CIEL ADAPTATION: Billing & Burn Ops (The Value Layer)

This skill formalizes the auditing of system costs and revenue integrity. it prioritizes code-backed truth and identifying high-burn failure patterns.

## Burn Auditing (ECC Tools)
1. **Trace Ingress**: Map every webhook -> queue -> worker path before suggesting fixes.
2. **Recursion Risk**: Treat app-generated branches/PRs as priority-0 recursion risks.
3. **Premium Leakage**: Verify if capped/free users are hitting premium analyzers (e.g., Opus).
4. **Retry Burn**: Fix deterministic failures that waste tokens on repeated non-transient errors.

## Revenue & Pricing Truth
- **Paid Sales**: Rely ONLY on live billing data or timestamped snapshots.
- **Product Truth**: Inspect the entitlement code path. PROHIBIT assuming "team billing" exists just because it's in the marketing copy.
- **Customer Impact**: Distinguish between accidental duplicates and deliberate multi-seat intent before refunding.

## Decision Format
Every audit MUST end with:
- **Snapshot**: Timestamped revenue/anomalies.
- **Product Truth**: What the code actually does vs marketing claims.
- **Decision**: Refund, Preserve, or No-op.
- **Gap**: The exact backlog item required to fix the workflow.

## Anti-Patterns
- **Diagnosis via Marketing**: Claiming a feature works because it's on the landing page.
- **Refund-First**: Refunding without classifying the issue, losing valuable intent data.
- **Expensive Failure**: Spending $50 in tokens only to fail on a file-push conflict.
