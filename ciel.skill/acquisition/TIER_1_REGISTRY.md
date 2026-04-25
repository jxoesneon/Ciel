# TIER_1 — Curated Registry

Search known-good skill indexes before going to external sources.

## Known Registries

Populated via `SOURCES.md`. Examples:

- Official skills marketplace (Anthropic's, if/when public).
- Curated lists such as `awesome-claude-skills`, `awesome-gemini-skills`.
- Teammate / organization internal registries (configured per-project).
- Ciel's own previously-seen-and-accepted skills catalog.

## Query Protocol

1. Normalize gap description into a short query.
2. Hit each registry endpoint:
   - HTTP GET / JSON index.
   - Git repo clone + parse `index.md` if a curated repo.
   - MCP tool if a registry-as-MCP server is configured.
3. Rank matches by:
   - trigger overlap,
   - tag overlap,
   - license compatibility,
   - host runtime compatibility,
   - origin trust score.
4. Return top-K (default 3).

## Ranking Score

```text
score = 0.35 * tag_overlap

      + 0.25 * trigger_overlap
      + 0.15 * origin_trust
      + 0.15 * runtime_compat
      + 0.10 * license_compat

```

## Cutoff

If top score < `tier1_floor` (default 0.5), skip to Tier 2. If ≥ floor, attempt fetch.

## Fetch

`seed_skills/web_fetch/SKILL.md` retrieves the `.skill` file. Checksum verified against registry-provided hash where available.

## Fallthrough

On empty result or fetch failure, proceed to Tier 2. Log the failure mode for future registry source tuning.

## Trust

Tier 1 sources carry their own trust score in `SOURCES.md`. Council still runs but with Safety pre-biased higher starting point.
