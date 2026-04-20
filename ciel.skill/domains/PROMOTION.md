# PROMOTION — Local → Global Protocol

Full protocol for lifting a project-scoped learning to Ciel's global self.

## Entry Criteria

See `council/rubrics/PROMOTION_RUBRIC.md`. Minimum:

- recurrence across ≥ 2 projects,
- stability across ≥ N=10 invocations,
- non-leaking,
- non-conflicting with existing global rule.

## Pipeline

1. **Candidate identification** — `self_improvement/LOCAL_IMPROVEMENT.md` marks mature learnings.
2. **Generalization** — strip project identifiers, file paths, repo-specific jargon. Rendered form must be human-readable and self-contained.
3. **Cross-project check** — MemPalace semantic search across other project partitions for similar patterns (as evidence of universality).
4. **Conflict check** — does any existing global rule/skill contradict or cover this?
5. **Council via `PROMOTION.md` scope** — `council/invocation_scopes/PROMOTION.md`.
6. **Apply** — on pass, write to appropriate global subtree (skills, rules, prompts, etc.) with provenance metadata.
7. **Cleanup** — mark local learnings `promoted:true`.

## Provenance Metadata

```yaml
promoted_from:
  projects: ["<hash1>", "<hash2>"]     # hashed identifiers, never paths
  first_seen: 2026-01-03
  recurrences: 4
  run_id: <council run id>
```

Stored in the target global file's frontmatter (for skills) or in `~/.ciel/registry/promotions.json`.

## Reversibility

A promotion is a git commit; reversing is `git revert`. Post-revert, the learning reverts to `local` status in originating projects (if still present).

## Demotion

If a promoted learning proves too project-specific after global use, `council/invocation_scopes/SKILL_CONFLICT.md` may propose demoting it back to project scope(s). Rare.

## User Initiated

`/ciel-promote <learning_id>` runs the pipeline on-demand. Useful when a user knows a local pattern belongs globally.
