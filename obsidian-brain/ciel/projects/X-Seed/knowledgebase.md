---
title: X-Seed — Knowledgebase
project_note: knowledgebase
type: project-note
tags: [project, knowledgebase, X-Seed]
created: 2026-07-11
status: active
---

# X-Seed — Knowledgebase

Hub note for the X-Seed Android P2P stream aggregator and Stremio addon host.

## Summary

X-Seed (v1.0.0-rc.1) is an Android-native, Flutter-built P2P stream aggregator and Stremio addon host. It runs a full IPFS/libp2p node on Android, aggregates streams from built-in legal providers and community torrent providers, and exposes a local Stremio Addon SDK v3 server bound to `127.0.0.1:7979`. The app is split into `play` (Google Play, Firebase Analytics/Crashlytics) and `full` (GitHub sideload, zero backend) flavors. A Fly.io redirect proxy (`x-seed-forge.fly.dev`) solves Stremio's rustls TLS validation so users can install the addon via `stremio://` deep links.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/X-Seed` |
| Local path | `C:/Users/josee/X-Seed` |
| Version | `1.0.0-rc.1` |
| Visibility | PRIVATE |
| License | MIT |
| Flutter SDK | `3.44.4` / Dart `3.12.2` |
| Tests | **1735 passing, 0 failing** |
| `flutter analyze` | **0 issues** |
| Coverage | **89.05%** line coverage |

## Top-level structure

- `x_seed/` — main Flutter application.
- `docs/` — specs, ADRs, audits, plans, legal docs.
- `forge/` — Fly.io redirect proxy for Stremio addon installation.
- `play-store-assets/` — Google Play metadata and assets.
- `.agents/` — automated agent procedures and build instructions.
- `.github/workflows/` — CI/CD workflows.
- `Dockerfile`, `Makefile`, `README.md`, `CHANGELOG.md`, `LICENSE`.

## Architecture at a glance

```
X-Seed Android App
├── UI Layer (Flutter)
│   ├── Search / Browse / Watchlist / Status / Settings (5-tab shell)
│   ├── Detail screen with stream health, Stremio actions, subtitles
│   └── Onboarding + Stremio setup
├── Feature Layer
│   ├── Addon — local Stremio addon server (shelf, port 7979)
│   ├── Scraper — provider registry, community plugins, tracker optimization
│   ├── IPFS — libp2p node integration and adaptive governor
│   ├── Security — Ed25519 identity, Android Keystore, biometric gate
│   ├── Bridge — SQLite metadata/subtitle cache, OpenSubtitles proxy
│   └── Core — models, settings, normalization, blocklist
├── Service Layer
│   ├── Background service (flutter_background_service)
│   ├── WorkManager tasks (resurrection, tracker refresh, precache)
│   ├── Log service (rotating on-disk logs)
│   └── Sqflite / Hive / Secure storage
└── External
    ├── Stremio client (via addon protocol)
    ├── Forge proxy (Fly.io HTTPS → local HTTP redirect)
    └── Providers (NASA, Internet Archive, CC, Public Domain, community)
```

## Subsystem drill-down

- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|Addon / Stremio]] — local addon server, manifest, endpoints, deep links, Forge proxy.
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|Providers / Scraper]] — built-in providers, community plugin system, tracker optimization, DHT, stream health.
- [[ciel/projects/X-Seed/subsystems/ui-ux-routing.md|UI / UX / Routing]] — 5-tab shell, search, detail, settings, onboarding, localization, deep links.
- [[ciel/projects/X-Seed/subsystems/background-services.md|Background Services]] — foreground service, WorkManager, resurrection, precache, tracker refresh, logging.
- [[ciel/projects/X-Seed/subsystems/security-build-ci.md|Security / Build / CI]] — Ed25519 identity, Android Keystore, biometric gate, root detection, DMCA, flavors, CI/CD.
- [[ciel/projects/X-Seed/subsystems/ipfs-node.md|IPFS / libp2p Node]] — on-device node integration, adaptive governor, swarm service.

## Verification status

| Check | Command | Result |
|-------|---------|--------|
| Static analysis | `flutter analyze --fatal-infos` | 0 issues |
| Unit/widget tests | `flutter test` | 1735 passing, 0 failing |
| Coverage | `flutter test --coverage` | 89.05% line coverage |
| Debug APK (play) | `flutter build apk --debug --flavor play` | success |
| Debug APK (full) | `flutter build apk --debug --flavor full` | success |

## Recent changes (post-2026-07-09)

- **Stremio player deep links**: `stremio:///player/{encodedStream}` with zlib+base64+URL-encoded stream object containing `infoHash`, `sources`, and optional `name`.
- **Magnet / btih ID handling**: Addon server now URL-decodes IDs and the manifest declares `magnet` and `btih` in `idPrefixes`. `StreamIdParser` handles `magnet:?xt=urn:btih:HASH` and raw `btih:HASH`.
- **Catalog search routing**: Added `/catalog/<type>/<id>/<extra>.json` route to handle Stremio's path-segment search extras (`search=QUERY`).
- **Forge proxy**: Added `forge/` with HTTPS Fly.io deployment that redirects resource requests to the local HTTP server, enabling native `stremio://` addon install.
- **Community provider registry**: Migrated community providers into a compiled-in registry; 11 active, 6 disabled due to site issues.
- **Tracker optimization (Sprint 11)**: UDP scrape client, tracker scorer, tracker health monitor, DHT peer discovery, stream health service, tracker refresh service.
- **Port move**: Addon server default port moved from `11470` to `7979` (fallback `7980–7988`) to avoid collision with Stremio's own server.

## Recent architectural decisions

- [[ciel/kg/decisions/xseed-stremio-player-deep-link.md|Stremio player deep link format]]
- [[ciel/kg/decisions/xseed-magnet-btih-id-routing.md|Magnet / btih ID routing]]
- [[ciel/kg/decisions/xseed-catalog-search-path-segment.md|Catalog search as path segment]]
- [[ciel/kg/decisions/xseed-forge-redirect-proxy.md|X-Seed Forge redirect proxy]]

## Relationship to ecosystem

- **IPFS** — local path dependency on `dart_ipfs` from the sibling `IPFS` repo; runs full libp2p node on Android.
- **SeedSphere / Gardener** — X-Seed is a mobile-first port; `seedsphere-source/` is gitignored reference material.
- **Stremio** — X-Seed acts as a local addon and uses Stremio as an optional external player.

## Key files for deeper context

1. `x_seed/AGENTS.md` — verification commands, known issues, sprint architecture.
2. `README.md` — build instructions, sprint status, feature list.
3. `CHANGELOG.md` — release history.
4. `docs/specs/ARCHITECTURE.md` — system overview.
5. `docs/specs/ADDON_API_SPEC.md` — Stremio addon contract.
6. `docs/specs/SECURITY_SPEC.md` — keystore, biometric, Ed25519, key rotation.
7. `docs/specs/IPFS_NODE_SPEC.md` — Android libp2p node requirements.
8. `docs/specs/PROVIDER_SCRAPER_SPEC.md` — plugin-based scraping engine.
9. `docs/specs/UI_UX_SPEC.md` — mobile-first design system.
10. `docs/specs/BACKGROUND_SERVICE_SPEC.md` — foreground service specification.
11. `docs/specs/RELEASE_SPEC.md` — CI/CD, signing, Play Store, GitHub sideload.

## Related

- [[ciel/projects/X-Seed/X-Seed.md|X-Seed overview]]
- [[ciel/projects.md|Projects index]]
- [[ciel/projects/IPFS/IPFS.md|IPFS]] (local dependency)
- [[ciel/projects/SeedSphere/SeedSphere.md|SeedSphere]] (conceptual upstream)
- [[ciel/diary/2026-07-11-full-mine-xseed.md|This session's diary]]
