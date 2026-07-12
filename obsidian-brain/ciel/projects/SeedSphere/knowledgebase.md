---
title: SeedSphere — Knowledgebase
project_note: knowledgebase
type: project-note
tags: ["project-note","knowledgebase"]
status: active
created: 2026-07-09
updated: 2026-07-09
source: "https://github.com/jxoesneon/SeedSphere"
---

# SeedSphere — Knowledgebase

Synthesized expansion from the read-only subagent exploration of the local clone.

## Summary

SeedSphere is a decentralized, P2P-powered media discovery and streaming engine designed as a Stremio tracker-addon. The current “Federated Frontier” 2.0 ecosystem consists of a shared Dart core, a Shelf-based router/bootstrap node deployed on Fly.io, a Flutter Gardener client, a Cloudflare Worker bridge, and a web portal. It scrapes 14+ torrent/indexer providers, runs an IPFS/libp2p swarm, and supports debrid integrations (Real-Debrid, All-Debrid, Premiumize, Orion).

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/SeedSphere` |
| Local path | `C:/Users/josee/SeedSphere` |
| Versions | Gardener/Router 2.3.26, Core/Bridge 2.3.22 |
| Visibility | PUBLIC |
| License | MIT |
| Stars | 7 |

## Top-level structure

- `seedsphere_core/` — shared Dart library (scrapers, normalization, rate limiting, trackers).
- `router/` — Dart Shelf server; Docker + Fly.io deployment.
- `gardener/` — Flutter client app with local Stremio server.
- `bridge/` — Cloudflare Worker (TypeScript/Hono).
- `portal/` — web dashboard (HTML/JS/CSS).
- `.github/workflows/` — unified CI/CD, deploy, gardener CI, CodeQL.
- `README.md`, `CHANGELOG.md`, `ROADMAP.md`, `GARDENER_MAPPING.md`.

## Three-tier architecture

```
Gardener (Flutter)
  → local Stremio server (11470) / P2P Manager
      → Router (Dart Shelf on Fly.io: 8080 + libp2p bootstrap 4001)
          → Bridge (Cloudflare Worker with KV cache)
```

### seedsphere_core

- `metadata_normalizer.dart` — SxxEyy detection, title cleaning.
- `scraper_engine.dart` — parallel aggregator.
- 14+ scrapers: Torrentio, YTS, EZTV, Nyaa, 1337x, PirateBay, Torznab, Anidex, TokyoTosho, Zooqle, Rutor, Torlock, MagnetDL, TorrentGalaxy.
- `tracker_service.dart` — UDP tracker client.
- `rate_limiter.dart`, `user_agent_rotator.dart`.

### Gardener (Flutter client)

- Local Stremio addon server (`stremio_server.dart`) on port 11470.
- Stream resolver/aggregator, debrid providers, identity manager (Ed25519), config manager, local KMS.
- P2P manager background isolate using vendored `dart_libp2p/`.
- UI: Aetheric Glass theme, Riverpod state management.

### Router (Dart server)

- `bin/server.dart` — Shelf router with 20+ endpoints.
- Services: addon, scraper, swarm, auth (JWT/Ed25519), pairing, linking, tracker, mailer, health, db, p2p node.
- SQLite persistence; bootstrap node on Fly.io.

### Bridge (Cloudflare Worker)

- Hono framework; KV SWR caching.
- Routes: `/manifest.json`, `/stream/:type/:id`, `/subtitles/:type/:id`.
- Aggregates Torrentio + router swarm, normalizes metadata.

## Build / test / deploy

```bash
# Gardener
cd gardener
flutter pub get
flutter run
flutter analyze --no-fatal-infos
flutter test

# Router
cd router
dart pub get
dart run bin/server.dart   # port 8080
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
dart test

# Bridge
cd bridge
npm ci
npm run dev
npm test
npm run deploy

# Docker / Fly.io
docker build -f router/Dockerfile -t seedsphere-router .
flyctl deploy --config router/fly.toml --remote-only
```

CI:

- `unified-ci.yml` — dynamic matrix for gardener, router, bridge, legacy.
- `deploy-server.yml` — analyze/test pre-flight, deploy Router to Fly.io, Bridge to Cloudflare.
- `gardener-ci.yml` — Flutter multi-platform builds.
- `codeql.yml` — security scanning.

## Recent git state (manual snapshot)

- **Latest recorded release:** 2.3.0 (2026-03-16) zero-vulnerability milestone.
- **Working tree:** clean in read-only snapshot.
- **Recent commits:**
  - `effbf1e` chore(rules): add Stremio Web streaming and compatibility guidelines
  - `eefb630` fix(router): resolve Stremio Web mixed content and data URI blocks with secure HTTPS mp4 streams
  - `979a380` feat(router): add playable mock stream fallback for user testing
  - `837bd9c` fix(infra): absolute final synchronization of imports and parameters
  - `34ebc00` fix(infra): use super parameters to pass strict CI analyzer

## Security features

- Ed25519 signing for critical communications.
- XChaCha20-Poly1305 for AI provider keys.
- SSRF and ReDoS mitigation, rate limiting, CSRF.
- Secure storage via `flutter_secure_storage`.

## Known limitations

- Full UDP BitTorrent scraping deferred.
- Provider failover not yet implemented.
- Advanced scoring weights need refinement.
- Tracker health validation pending.

## Key files for deeper context

1. `README.md` — overview and quick start.
2. `ROADMAP.md` — strategic vision.
3. `GARDENER_MAPPING.md` — legacy vs new architecture mapping.
4. `seedsphere_core/lib/src/scraper_engine.dart` — scraper aggregation.
5. `router/bin/server.dart` — main server entry.
6. `gardener/lib/main.dart` — Flutter bootstrap.
7. `bridge/src/index.ts` — Cloudflare Worker.
8. `router/fly.toml` / `bridge/wrangler.toml` — deployment configs.
9. `.github/workflows/unified-ci.yml` — dynamic CI.
10. `CHANGELOG.md` — detailed version history.

## Related

- [[ciel/projects/SeedSphere/SeedSphere.md|SeedSphere overview]]
- [[ciel/projects.md|Projects index]]
- [[ciel/projects/X-Seed/X-Seed.md|X-Seed]] (mobile descendant)
- [[ciel/projects/IPFS/IPFS.md|IPFS]] (P2P dependency)
