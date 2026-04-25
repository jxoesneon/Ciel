---
name: healthcare-systems
version: 1.0.0
format: skill/1.0
description: EMR/EHR encounter workflows and Clinical Decision Support (CDSS) engine patterns.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(build|design).*(emr|ehr|patient encounter|cdss|interactions|dosing)"
    confidence: 0.95
  - pattern: "healthcare safety patterns"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Healthcare Systems (Safety & Workflows)

This skill formalizes the development of patient-critical healthcare software. It mandates fail-safe logic and accessibility-first UI.

## Encounter Flow (Vertical Stack)
The clinical encounter MUST be a single-page vertical scroll to prevent context loss:
1. **Sticky Header**: Allergies and active meds ALWAYS visible.
2. **Structured Entry**: Complaint -> History -> Exam -> Vitals.
3. **CDSS Gate**: Interaction and Dose checks before signing.
4. **Sign & Lock**: Edits prohibited after signing; use Addendums only.

## CDSS Engine (Pure Functions)
The Decision Support engine MUST be a side-effect-free library:
- **Interactions**: Bidirectional checks (Drug A vs B AND B vs A).
- **Dosing**: Block (not pass) if weight is missing for mg/kg drugs.
- **Scoring**: Exact implementation of clinical scales (e.g., NEWS2).

## Alert Severity & UI
- **Critical**: Non-dismissable modal. Must document override reason in audit trail.
- **Major**: Inline orange banner. Acknowledge required.
- **Standard**: No auto-dismissing toasts for clinical data.

## Anti-Patterns
- **Silent Failures**: Allowing a drug search to pass if the interaction DB is unreachable.
- **Amnesiac Overrides**: Overriding a critical block without storing the reason in the audit trail.
- **LocalStorage PHI**: Storing patient data in browser storage (creates HIPAA/GDPR risk).
