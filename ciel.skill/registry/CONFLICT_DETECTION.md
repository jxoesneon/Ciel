# CONFLICT_DETECTION

Ongoing detection of overlapping or drifting skills in the live registry.

## Detection Signals

| Signal | Check |
| --- | --- |
| direct trigger overlap | two skills share any trigger token at identical normalization |
| functional overlap | identical `io_contract.input.shape` + identical `side_effects` + similar description (cosine sim > 0.85) |
| drift | multiple versions of the same `id` exist on disk without a linear history (git log anomaly) |
| shadowing | one skill's triggers are a strict subset of another's, and its tags are a subset |
| cross-tier duplication | a Tier 0 seed and a Tier 2/3 acquired skill cover the same capability |

## Scanning

- **Live** — at each registry mutation, incremental scan against neighbors.
- **Scheduled** — full sweep every `registry.config.coherence_sweep_interval` (default: weekly).
- **On-demand** — `/ciel-registry --check` triggers a full scan.

## Output

Conflicts recorded in `~/.ciel/registry/conflicts/<run_id>.json`:

```json
{
  "run_id": "...",
  "conflicts": [
    {
      "type": "functional_overlap",
      "skills": ["a", "b"],
      "similarity": 0.91,
      "suggested_resolution": "merge"
    }
  ]
}
```

## Handling

All detected conflicts flow to `council/invocation_scopes/SKILL_CONFLICT.md`. Ciel does not auto-merge; all merges are Council-gated.

## Known-Safe Overlap

Some overlap is intentional (e.g. a generic `shell` skill and a specialized `docker` skill both match `"run container"` loosely). Coherence member marks these with `expected_overlap: true` in the registry so future scans ignore them.
