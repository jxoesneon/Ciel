# Contributing to CIEL 1.0

Thank you for your interest in contributing to CIEL! As a high-integrity partner intelligence, we maintain strict standards for all contributions.

## ⚖️ The CIEL Standard

Every contribution MUST align with the CIEL 1.0 architectural mandates:

1. **The Iron Law**: No feature or fix is complete without fresh verification evidence (logs, screenshots, or test results).
2. **TDD 80%**: All logic changes MUST include baseline tests, and we target a minimum of 80% code coverage.
3. **No Placeholders**: We do not accept code with `// TODO`, `...`, or other omission markers. All contributions must be "Reasoning Dense" and complete.
4. **Guild Alignment**: New agents or skills should be mapped to one of the 10 Elite Guilds or 140 harmonized frameworks.

## 🛠️ Development Workflow

1. **Research**: Always perform a deep audit of the existing `skills/` and `agents/` to prevent redundancy.
2. **Harmonization**: If adding a new capability, ensure it is harmonized into a dense framework rather than added as a narrow, disparate tool.
3. **Platform-agnostic agent surface**: Agent-facing scripts are single-source cross-platform — no per-OS policy logic. Per-OS shims may exist only as thin dispatchers that forward arguments and exit codes unchanged to the shared core (see `ciel.skill/architecture/ADR_20261003_PLATFORM_AGNOSTIC_AGENT_SCRIPTS.md`).
4. **Verification**: Run the project's verification loops before submitting your proposal.

## 🏛️ Council Review

All non-trivial changes will be reviewed by the **Council of Five** (Capability, Coherence, Safety, Efficiency, Evolution). A Safety veto is absolute.

---
*CIEL is more than a project; it is a shared intelligence. Thank you for helping her grow.*
