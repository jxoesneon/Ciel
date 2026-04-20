# TOKEN_CACHE — Gemini CLI

Gemini CLI supports token caching at the API layer. Ciel uses it similarly to Claude Code's prompt cache.

## Cacheable Segments

| Segment | TTL | Invalidation |
| --- | --- | --- |
| `SKILL.md` + `core/*` | 1h | mutation |
| Registry L0 bundle | 1h | reg changes |
| Council personas | 1h | council/members change |
| Extension playbooks | per-extension | extension update |

## Strategy

Place cacheable content at the head of the system prompt so the prefix is deterministic. Runtime-specific cache keys include the Gemini model id to avoid cross-model pollution.

## Measurement

```json
{ "cache_hit_tokens": ..., "cache_miss_tokens": ..., "cache_ratio": ... }
```

## Cost Impact

Ciel's router budget assumes cache warms after N=2 calls. Cold-start windows (session start, post-mutation) cost more; Ciel respects this by not firing multiple Council rounds back-to-back after a mutation unless necessary.
