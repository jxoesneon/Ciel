# SCOPE — SKILL_INTEGRATION

Triage: incoming acquired skill needs to join the registry.

## Trigger

`acquisition/ACQUISITION.md` has produced a harmonized candidate. Invoke the Council.

## Preamble Injected

Each member's Stage 1 prompt is prefixed:

> You are evaluating a **newly-acquired skill** for integration into Ciel's global registry. Source tier: <1|2|3>. Origin: <url|mcp|registry_id>. Harmonization report attached. Apply your lens.

## Inputs

- candidate skill (L1 default, L2 on request),
- harmonization diff,
- trust model result from `acquisition/TRUST_MODEL.md`,
- sandbox execution trace (from `acquisition/SANDBOX.md`) if Tier 2/3,
- registry metadata slice for neighbors (Capability overlap check).

## Stage 1 Output Expected

All five lenses score per `rubrics/SCORING.md`. Safety must explicitly note sandbox trace assessment.

## Stage 2

Anonymized cross-review.

## Stage 3

Chairman synthesizes. On pass, skill is:

- installed via `seed_skills/skill_installer/SKILL.md`,
- registered in `registry/REGISTRY.md`,
- committed to `~/.ciel/` with message `skill_integration: add <skill_id> (tier <n>)`,
- tagged in MemPalace `ciel/registry/<skill_id>`.

## Reject Path

- Tier 3 reject → skill discarded, origin logged, lower trust bias on that source.
- Tier 2 reject → MCP server is not auto-added; noted in acquisition.config.
- Tier 1 reject → rare; surfaces to user as a registry curation issue.
