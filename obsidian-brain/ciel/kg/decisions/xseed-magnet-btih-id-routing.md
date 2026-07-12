---
title: X-Seed magnet / btih ID routing in addon server
type: decision
project: X-Seed
tags: [decision, x-seed, addon, magnet, btih, id-prefix]
status: accepted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Decision: X-Seed magnet / btih ID routing in addon server

## Status

Accepted

## Context

X-Seed addon server needs to handle Stremio requests for torrent content. Stremio IDs can arrive as:

- `magnet:?xt=urn:btih:HASH` (full magnet URI)
- `btih:HASH` (raw info-hash prefix)
- URL-encoded variants of the above when they appear as path segments (`magnet%3A%3Fxt%3Durn%3Abtih%3A...`)

Without proper handling, these IDs were routed incorrectly, causing 404s or failed meta/stream lookups. The addon manifest previously only declared `idPrefixes: ['tt', 'kitsu']`, so Stremio did not route magnet IDs to X-Seed at all.

## Decision

1. Change the canonical ID format from `btih:HASH` to `magnet:?xt=urn:btih:HASH` for Stremio-facing IDs.
2. Add `magnet` and `btih` to `idPrefixes` in both the local addon manifest and the Forge manifest.
3. Update `StreamIdParser` to recognize both `magnet:?xt=urn:btih:HASH` and raw `btih:HASH`.
4. URL-decode incoming IDs in the addon server handlers (`catalog`, `meta`, `stream`) before parsing.
5. Update the addon server `metaHandler` to extract the info hash from both formats via `extractInfoHash`.
6. Update `ProviderAggregator` to handle magnet IDs directly.
7. Update all torrent providers to emit `magnet:?xt=urn:btih:HASH` as the canonical ID.

## Consequences

- **Positive**: Stremio now routes `magnet` and `btih` IDs to the X-Seed addon.
- **Positive**: URL-encoded magnet IDs in path segments are correctly decoded and parsed.
- **Positive**: `meta`, `stream`, and `catalog` handlers all support torrent IDs.
- **Trade-off**: Requires URL decoding on every ID-bearing path segment; adds minimal overhead.
- **Trade-off**: Providers must be audited to ensure they emit the canonical magnet format.

## Related

- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|X-Seed — Addon / Stremio]]
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|X-Seed — Providers / Scraper]]
- `x_seed/lib/src/features/addon/id_parser.dart`
- `x_seed/lib/src/features/addon/addon_server.dart`
- `x_seed/lib/src/features/addon/provider_aggregator.dart`
- `x_seed/lib/src/features/addon/meta_service.dart`
- `forge/server.dart`
