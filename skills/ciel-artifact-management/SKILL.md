---
name: ciel-artifact-management
description: CIEL's framework for isolated, structured storage of plans, audits, and ephemeral reasoning data.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Artifact Management (The Vault)

This skill formalizes the "Antigravity" model of separating product code from agentic waste. It ensures that transient reasoning data is stored in isolated, auditable vaults.

## Artifact Taxonomy

1. **Plans**: Implementation blueprints, task lists, and migration strategies.
2. **Audits**: Security reports, spec-compliance checks, and adversarial reviews.
3. **Visuals**: Screenshots, terminal recordings, and diagram exports.
4. **Transient**: 'Noise' reduction dumps, intermediate trace data, and session checkpoints.

## Storage Protocol

- **Location**: All artifacts MUST be stored in `~/.ciel/artifacts/` or `.ciel/artifacts/`.
- **Naming**: `[type]_[date]_[slug].md` (e.g., `plan_20260422_db-refactor.md`).
- **Isolation**: Artifacts are never committed to the primary `src/` or `lib/` directories.
- **Reference**: High-level summaries belong in `CHANGELOG.md`; deep logic belongs in `artifacts/`.

## The Feedback Loop

- **Review Policy**: High-risk plans (CRITICAL in `risk/`) require an explicit artifact review before execution.
- **Verification**: Every 'Iron Law' claim must reference an artifact in the `audits/` or `visuals/` vault.

## Anti-Patterns

- **Repo Pollution**: Saving planning files or log dumps in the project root.
- **Amnesiac Execution**: Deleting a plan before the implementation is fully verified.
- **Opaque Reasoning**: Modifying critical system logic without a stored artifact.
