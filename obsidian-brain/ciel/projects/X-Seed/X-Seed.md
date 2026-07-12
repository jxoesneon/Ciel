---
title: X-Seed
project_note: hub
type: project
tags: [project, X-Seed]
created: 2026-07-11
status: active
---

# X-Seed

Android-native P2P stream aggregator and Stremio addon host powered by IPFS/libp2p.

## Metadata

| Field | Value |
|-------|-------|
| Owner | jxoesneon |
| Repository | https://github.com/jxoesneon/X-Seed |
| Homepage | — |
| Default branch | main |
| Primary language | Dart |
| Visibility | PRIVATE |
| License | MIT License |
| Stars | 0 |
| Forks | 0 |
| Created | 2026-06-29T00:33:13Z |
| Updated | 2026-07-12T00:00:00Z |
| Archived | false |
| Fork | false |

## Topics

android, dart, flutter, ipfs, libp2p, p2p, stremio, torrent

## Use and scope

X-Seed is a self-contained Android app that runs a full IPFS/libp2p node, aggregates streams via a user-configurable provider plugin system, and exposes a local Stremio addon endpoint directly from the phone. It ships in two flavors: `play` (Google Play compliant with Firebase Analytics/Crashlytics) and `full` (GitHub sideload, zero backend). A Fly.io redirect proxy (`x-seed-forge.fly.dev`) enables public addon installation via `stremio://` deep links.

## Local clone snapshot

- Path: `C:/Users/josee/X-Seed`
- Version: **1.0.0-rc.1**
- Status: Sprint 10 complete; recent work on Stremio `btih` player deep links, addon catalog/search routing, magnet ID support, community provider registry, tracker optimization (Sprint 11), search filtering, search relevance ranking, detail-screen UX P0-P2 fixes, and episode-specific stream filtering wiring (2026-07-12).
- Working tree: extensive modified and untracked files across docs, lib, tests, Android manifests, and Forge proxy code.
- Tests: **1,754 passing, 0 failing** as of 2026-07-12.
- `flutter analyze`: **0 issues** (0 errors, 0 warnings, 0 infos).
- Coverage: **89.05%** line coverage (last recorded run).
- Build: debug APKs for both flavors build successfully on the Windows dev box.
- Forge: deployed at `https://x-seed-forge.fly.dev`.

## Expanded knowledge

- [[ciel/projects/X-Seed/knowledgebase.md|X-Seed — Knowledgebase]] (hub)
- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|X-Seed — Addon / Stremio]]
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|X-Seed — Providers / Scraper]]
- [[ciel/projects/X-Seed/subsystems/ui-ux-routing.md|X-Seed — UI / UX / Routing]]
- [[ciel/projects/X-Seed/subsystems/background-services.md|X-Seed — Background Services]]
- [[ciel/projects/X-Seed/subsystems/security-build-ci.md|X-Seed — Security / Build / CI]]
- [[ciel/projects/X-Seed/subsystems/ipfs-node.md|X-Seed — IPFS / libp2p Node]]

## Recent architectural decisions

- [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md|Design Council review: detail screen UX]]
- [[ciel/kg/decisions/xseed-search-relevance-ranking.md|Search result ranking: relevance-based default sort]]
- [[ciel/kg/decisions/xseed-search-relevance-adult-filtering.md|Search result filtering: relevance + adult content]]
- [[ciel/kg/decisions/xseed-stremio-player-deep-link.md|Stremio player deep link format (`stremio:///player/{encodedStream}`)]]
- [[ciel/kg/decisions/xseed-magnet-btih-id-routing.md|Magnet / btih ID routing in addon server]]
- [[ciel/kg/decisions/xseed-catalog-search-path-segment.md|Catalog search as URL path segment]]
- [[ciel/kg/decisions/xseed-forge-redirect-proxy.md|X-Seed Forge HTTPS→HTTP redirect proxy]]

## Recent updates

- [[ciel/projects/X-Seed/updates/2026-07-11-search-relevance-ranking.md|Search relevance ranking]] — implemented `SearchSort.relevance` as default; core titles now rank above secondary mentions and release-group prefixed releases.
- [[ciel/projects/X-Seed/updates/2026-07-12-search-adult-relevance-filtering.md|Search adult/relevance filtering]] — fixed unrelated and adult content in search; filters and sorting verified correct.
- [[ciel/projects/X-Seed/updates/2026-07-11-physical-device-stremio-verification.md|Physical-device Stremio stream launch verification]] — partial success; Stremio deep link opens on phone, full X-Seed UI tap-to-play blocked by Flutter automation limits.
- [[ciel/diary/2026-07-12-detail-screen-p0-implementation.md|Detail screen P0 UX fixes]] — wired `StreamFilterBar`, removed misleading bottom Copy Magnet, added semantic labels and disabled-state tooltips; 1751 tests pass.
- [[ciel/diary/2026-07-12-detail-screen-px-implementation.md|Detail screen P1/P2 UX fixes]] — health label expansion, series episode selector, non-IMDb subtitle empty state, chip touch targets, dynamic type scaling, bottom-action hierarchy, skeleton health loading, polished subtitle preview, watchlist undo, content-type badge; 1751 tests pass.
- [[ciel/projects/X-Seed/updates/2026-07-12-detail-episode-filtering.md|Detail episode filtering wiring]] — episode selector now refetches episode-specific streams; season/episode metadata added to provider streams; health sparkline now accessible; 1754 tests pass.

## Related

- [[ciel/projects.md|Projects index]]
- [[ciel/projects/IPFS/IPFS.md|IPFS]] (local dependency)
- [[ciel/projects/SeedSphere/SeedSphere.md|SeedSphere]] (conceptual upstream)
- [[ciel/diary/2026-07-11-full-mine-xseed.md|Full mine diary]]
- [[ciel/diary/2026-07-11-xseed-stremio-phone-verification.md|Phone verification diary]]
- [[ciel/diary/2026-07-12-xseed-search-filtering.md|Search filtering diary]]
- [[ciel/diary/2026-07-11-search-relevance-ranking.md|Search relevance ranking diary]]
- [[ciel/diary/2026-07-12-detail-screen-p0-implementation.md|Detail screen P0 implementation diary]]
- [[ciel/diary/2026-07-12-detail-screen-px-implementation.md|Detail screen P1/P2 implementation diary]]
- [[ciel/diary/2026-07-12-detail-episode-filtering-wiring.md|Detail episode filtering wiring diary]]
