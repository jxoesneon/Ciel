# GLOBAL_IMPROVEMENT

Improvement loop targeting `~/.ciel/`.

## What Changes Here

- Core identity files (via Constitutional amendment).
- Router weights, path floors.
- Council rubrics (non-locked portions).
- Skills in the global registry: add, update, deprecate, remove.
- Adapters: update existing, build new.
- Seed skills: tuning.
- Acquisition sources: trust adjustments.

## Flow

1. Aggregated signals produce a proposal candidate.
2. `prompts/self_improvement/improvement_proposal.md` drafts a diff + rationale.
3. Proposal categorized: `trivial | standard | structural | constitutional`.
4. Category determines gating:
    - `trivial` — config fine-tuning inside defaults. Auto-apply with activity log entry. Example: slight adjustment of router cache TTL.
    - `standard` — functional changes to non-locked files. Council-gate via `SELF_MODIFICATION.md`.
    - `structural` — changes touching multiple components. Full Council + wider watch window.
    - `constitutional` — touches locked files. Council + explicit user confirmation.
5. Apply → git commit → `REGRESSION_DETECTION.md` watch.

## Commit Conventions

See `registry/VERSIONING.md`. All commits reference the Council run id and the originating trigger(s).

## Example Loop

- Trigger: `route_miss` rate 25% for `docker:*` tag over last 50 routings.
- Proposal: add fast-path trigger alias `container` to `docker/SKILL.md`.
- Category: standard.
- Council: passes.
- Applied.
- Watch: miss rate drops to 4% over next 30 routings → improvement confirmed.

## Rate Limiting

`improvement.config.global_max_per_day` (default 20) caps applied changes. Backlog queues.
