---
name: healthcare-compliance
version: 1.0.0
format: skill/1.0
description: CIEL's framework for HIPAA (US), GDPR (EU), and PHI/PII compliance in healthcare environments.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(healthcare|hipaa|phi|pii|baa).*(compliance|patient|clinician)"
    confidence: 0.95
  - pattern: "is this (healthcare|hipaa) compliant"
    confidence: 0.9
---

# CIEL ADAPTATION: Healthcare Compliance (PHI/PII Protection)

This skill formalizes the handling of Protected Health Information (PHI) and Personally Identifiable Information (PII). It acts as an overlay to `security-review` for regulated medical environments.

## The Three Pillars
1. **Classification**: Identify PHI (names, DOB, national IDs, medical records) and PII (clinician payouts, staff bank details).
2. **Access Control**: Enforce Row-Level Security (RLS) and scoped authorization.
3. **Audit**: Maintain a tamper-proof log of all PHI access events.

## Non-Negotiable Guardrails
- **Zero-PHI Logs**: Never place patient data in logs, analytics, or error strings.
- **UUIDs Only**: Use opaque internal IDs (UUIDs) in URLs and storage. Never use MRNs or names as keys.
- **BAA Check**: Verify that third-party vendors (LLM providers, SaaS) are covered by a BAA before transmitting data.
- **Browser Isolation**: Keep PHI in-memory only. Do NOT use `localStorage`.

## Operational Workflow
- **Diagnostic**: Is the data PHI? Is the vendor BAA-approved?
- **Review**: Use an Auditor sub-agent with the "Healthcare Reviewer" role for clinical decision logic.
- **Verification**: Run `verification-loop` with negative tests for cross-facility isolation.

## Anti-Patterns
- **Leaky Errors**: "Patient John Doe not found in Facility X" (leaks name and facility).
- **Service Role Abuse**: Using the `service_role` key in client-side code, bypassing RLS.
- **Blind Promotion**: Promoting un-redacted clinical summaries to the global `MemPalace`.
