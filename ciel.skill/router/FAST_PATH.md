# FAST_PATH

Deterministic registry lookup. O(1) key match plus O(log n) tag search.

## Inputs

- `request.text` (normalized, lowercased, tokenized).
- `request.project_ctx.tags` (from project analyzer).
- `request.history_ptr` (recent skills-used bias).

## Lookup Order

1. **Exact trigger match** — every skill's `SKILL.md` frontmatter has `triggers: [...]`. Match against normalized request tokens. Exact hit → candidate.
2. **Tag intersection** — `registry/INDEXING.md` maintains an inverted tag index. Rank candidates by tag-match cardinality × usage-recency weight.
3. **Input-contract match** — `registry/SCHEMA.md` defines input contracts per skill. Filter candidates by contract compatibility with the current request shape.
4. **Confidence score** — compute:

```text
confidence = 0.4 * trigger_match

           + 0.3 * tag_intersection
           + 0.2 * contract_compat
           + 0.1 * recency_bias

```

## Floor

`router.config.fast_path_floor` (default `0.80`). Below floor → fall through.

## Ambiguity Resolution

If two candidates both clear the floor within 0.05:

- prefer the one with higher historical success on this host's project context,
- if still tied, prefer the leaner skill (smaller unpacked byte size),
- if still tied, hand to `REASONING_PATH.md` as a composition problem.

## Cache

- In-memory LRU of last 64 `(request_hash → skill_id)` pairs for the current session.
- Cache is not persisted; process-local to the host runtime.
- `router.config.cache_ttl` overrides default 1h in-session TTL.

## Telemetry

Every fast-path attempt logs:

```json
{ "path": "fast", "confidence": 0.87, "candidate": "git/SKILL.md", "hit": true, "ms": 12 }
```

which feeds `router/ROUTE_REGISTRY.md` for self-update signals.

## Failure Semantics

A fast-path miss is **not** an error. It is normal and triggers fall-through. Errors arise only on index corruption — detected by `registry/REGISTRY.md` periodic sweep.
