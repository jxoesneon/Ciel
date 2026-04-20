# PROMPT_CACHE — Claude Code

Claude Code supports prompt caching with 1h TTL. Ciel uses this aggressively to keep her context-heavy components cheap.

## What Ciel Caches

| Segment | TTL | Invalidation |
| --- | --- | --- |
| `SKILL.md` + `core/*` | 1h | any mutation to these files |
| Registry L0 bundle | 1h | any registration/deregistration |
| Council member personas | 1h | any change under `council/members/` |
| Active adapter sections | 1h | adapter config change |
| Prompts library | 1h | any change under `prompts/` |

## Strategy

Cacheable segments are placed at the **start** of the system prompt so the cache prefix is deterministic. Dynamic fields (current project context, live registry entries) come *after* the cacheable block.

## Invalidation

Ciel stamps a `cache_version` in `configuration/global/router.config.md`. On any write to a cacheable path, the stamp is bumped, forcing a fresh cache fill. This is a single atomic update rather than per-key invalidation.

## Measurement

Every request logs:

```json
{ "cache_hit_tokens": 12400, "cache_miss_tokens": 1800, "cache_ratio": 0.87 }
```

Drop below `router.config.prompt_cache.floor` → self-improvement trigger to inspect cache layout.

## Budget

Prompt cache reduces effective cost by ~90% on warm calls. The router's budget formula assumes warm cache after the first N=2 calls of a session; cold-start calls are budgeted at uncached rates.
