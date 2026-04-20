# CONFLICT_RUBRIC

Scoring for registry overlap / coherence conflict resolution.

## Triggering Scope

`invocation_scopes/SKILL_CONFLICT.md`. Invoked when `registry/CONFLICT_DETECTION.md` flags two registered skills with overlapping capability, or when a periodic `registry/COHERENCE_SWEEP.md` surfaces drift.

## Conflict Types

| Type | Example |
| --- | --- |
| Direct overlap | two skills match the same fast-path trigger |
| Functional overlap | different triggers but identical I/O contract and execution |
| Drift | a single skill has mutated into two incompatible variants across commits |
| Shadowing | one skill is strictly a subset of another |

## Resolution Options

- **Merge** — combine into one skill; retire the other.
- **Delegate** — keep both, split responsibilities via a routing rule.
- **Deprecate** — keep the superior, mark the other deprecated for removal.
- **Split** — neither is right; split into cleaner pieces.

## Per-Lens Scoring

| Lens | Asks |
| --- | --- |
| Coherence | Which option yields the tidier registry? |
| Capability | Which preserves the most unique capability? |
| Safety | Does the option introduce new risk vectors? |
| Efficiency | Which option reduces L0/L1 footprint most? |
| Evolution | Which option positions Ciel for future growth? |

## Decision

Chairman picks the option with majority lens support. Ties resolved by Efficiency lens (simpler registry wins).

## Actions

Chosen option is executed with:

1. A git commit with `CONFLICT_RESOLUTION:` prefix.
2. A `~/.ciel/conflicts/<run_id>.json` artifact capturing before/after state.
3. Updated `registry/REGISTRY.md` entries.
4. `CHANGELOG.md` entry under `### Changed`.

## Rollback

Git revert the conflict-resolution commit restores the prior state. Ciel will not auto-redo the same resolution on the next sweep unless new evidence justifies it.
