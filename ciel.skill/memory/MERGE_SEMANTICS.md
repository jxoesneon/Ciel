# MERGE_SEMANTICS

How local and global knowledge combine. Who wins on conflict. How promotion works.

## Priority

On any decision that reads both scopes:

1. **Local wins** for project-scoped values (rules, conventions, overrides).
2. **Global wins** for identity, constitution, locked invariants.
3. **Local reads global** via explicit `lift(key)` call — logged.
4. **Global reads local** via `with_project(id)` scope — logged.

## Conflict Resolution

| Conflict | Resolution |
| --- | --- |
| Local rule contradicts global rule (non-safety) | Local wins in this project |
| Local rule attempts to weaken a Safety-veto condition | Rejected; local overrides are capped by Constitution |
| Local skill with same id as global skill | Local shadows global in this project; global remains default elsewhere |
| Project escalation override below Constitutional floor | Rejected; fall back to floor |

## Promotion (local → global)

See `council/invocation_scopes/PROMOTION.md` + `council/rubrics/PROMOTION_RUBRIC.md`.

Mechanically:

1. `self_improvement/LOCAL_IMPROVEMENT.md` identifies mature learnings.
2. Generalization pass strips project identifiers.
3. Council of Five evaluates with elevated thresholds.
4. On pass, learning is written to `~/.ciel/` in the appropriate subtree and removed (or marked `promoted`) locally.
5. Provenance stored: which projects contributed.

## Demotion (global → local)

Rare. If a global rule proves project-specific, Council may:

- duplicate it to the specific project's `.ciel/`,
- mark the global version deprecated or narrowed.

Demotion requires Chairman synthesis + user notification.

## Activity Log

Every merge decision is logged:

```json
{ "kind": "merge", "op": "lift", "key": "...", "winner": "local", "reason": "..." }
```
