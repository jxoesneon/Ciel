# SCOPE — SKILL_CONFLICT

Triage: registry overlap or drift detected between two or more already-registered skills.

## Trigger

- `registry/CONFLICT_DETECTION.md` flags overlap, OR
- `registry/COHERENCE_SWEEP.md` periodic scan returns drift, OR
- Coherence member flagged `naming_conflict` in a prior Council run.

## Preamble

> You are evaluating a **conflict** between registered skills. Skills in conflict: `<id_a>`, `<id_b>`. Conflict type: `<direct|functional|drift|shadowing>`. Evidence attached. Apply your lens to pick a resolution.

## Inputs

- Both (or all N) conflicting skills at L1.
- Usage histograms from `router/ROUTE_REGISTRY.md`.
- Conflict type and diff.
- `rubrics/CONFLICT_RUBRIC.md` summary.

## Stage 1 Output Expected

Each member scores all resolution options (merge / delegate / deprecate / split) per `CONFLICT_RUBRIC.md`.

## Stage 3

Chairman selects the resolution option with weighted support. Safety retains veto over any resolution that would introduce risk.

## Execution

Chairman commits the resolution with message `conflict_resolution: <option> on (<ids>)`. Registry is updated atomically; both git and MemPalace reflect the new state.

## Preservation

Removed skill files are moved to `~/.ciel/.attic/<ts>/<skill_id>/` (retained, not deleted) for possible recovery via git.
