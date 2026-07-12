---
title: X-Seed — Providers / Scraper
project_note: subsystem
type: project-note
project: X-Seed
tags: [subsystem, x-seed, providers, scraper, tracker, dht]
status: active
created: "2026-07-12T07:44:58.464Z"
---

# X-Seed — Providers / Scraper

Stream aggregation layer: built-in legal providers, community provider registry, plugin system, and the tracker optimization system (Sprint 11).

## Summary

The Providers/Scraper subsystem is a dual-layer architecture: built-in legal video providers (compiled into the binary) and community providers (loaded from JSON configs or registered via a compiled-in community registry). It includes a sophisticated tracker optimization system with UDP/HTTP/DHT parallel scraping, Bayesian Average + EWMA scoring, active health monitoring, and DHT peer discovery. Streams are enriched with real-time peer counts, deduplicated, sorted, and filtered through the DMCA blocklist before being handed to the addon layer.

## Key files

| File | Purpose |
|------|---------|
| `x_seed/lib/src/features/scraper/scraper_manager.dart` | Parallel search orchestration with circuit breaker and cache |
| `x_seed/lib/src/features/scraper/providers/creative_commons_provider.dart` | Built-in CC-licensed video search |
| `x_seed/lib/src/features/scraper/providers/internet_archive_provider.dart` | Built-in archive.org provider (direct URLs + torrent/magnet) |
| `x_seed/lib/src/features/scraper/providers/nasa_provider.dart` | Built-in NASA public domain video library |
| `x_seed/lib/src/features/scraper/providers/public_domain_provider.dart` | Built-in public domain movies from archive.org |
| `x_seed/lib/src/features/scraper/providers/community/community_provider_registry.dart` | Compiled-in community provider registry |
| `x_seed/lib/src/features/scraper/community_plugin_config.dart` | JSON config model for user plugins |
| `x_seed/lib/src/features/scraper/community_plugin_loader.dart` | Loads plugins from filesystem directory |
| `x_seed/lib/src/features/scraper/dht_peer_discovery.dart` | BEP-5 DHT peer discovery |
| `x_seed/lib/src/features/scraper/magnet_enhancer.dart` | Appends public trackers to magnet links |
| `x_seed/lib/src/features/scraper/stream_health_service.dart` | Enriches streams with peer counts |
| `x_seed/lib/src/features/scraper/stream_resolver.dart` | Resolves CIDs to IPFS gateways, extracts web seeds |
| `x_seed/lib/src/features/scraper/tracker_scorer.dart` | Bayesian Average + EWMA tracker scoring |
| `x_seed/lib/src/features/scraper/tracker_health_monitor.dart` | Active probing with exponential backoff |
| `x_seed/lib/src/features/scraper/udp_scrape_client.dart` | BEP-48 UDP tracker scrape protocol |
| `docs/specs/PROVIDER_SCRAPER_SPEC.md` | Specification |

## Built-in providers

Four compiled-in providers implement `ContentProvider` with `isBuiltIn = true`:

- **CreativeCommonsProvider** — CC Catalog API, filters by video tags/extensions.
- **InternetArchiveProvider** — Advanced search + metadata endpoint; parses `.torrent` files for magnet generation.
- **NasaProvider** — NASA Images API; skips IMDb-style IDs (`tt` prefix) to avoid masking other providers.
- **PublicDomainProvider** — Archive.org `opensource_movies` collection with direct download URLs.

All built-in providers use `ScraperHttpClient` with rate limiting (3–5 sec intervals), 5-second timeout, and return empty lists on errors.

## Community providers

The `CommunityProviderRegistry` registers 11 active community providers:

- **Aggregators**: Torrentio, YTS, EZTV
- **Anime**: Nyaa, TokyoTosho
- **General Trackers**: PirateBay, X1337, TorrentGalaxy, RuTor
- **Open Media**: PublicDomainTorrents
- **Generic**: Torznab

Six providers are currently disabled due to site issues:

- Anidex (403 Cloudflare)
- MagnetDL (site down)
- Openverse (404)
- Torlock (JS-loaded magnets)
- Zooqle (error)

User-configured plugins can be added via JSON files in the `community_plugins/` directory (app external storage). Supported types:

- **Torznab** — Jackett/Prowlarr indexers (requires `apiUrl`, optional `apiKey`).
- **HTTP** — generic scrapers (requires `baseUrl`, optional `searchPath`, `streamPath`, `metaPath`).

The `CommunityPluginLoader` scans `.json` files, validates configs, and registers providers. Disabled plugins are skipped.

## Tracker optimization system (Sprint 11)

### Three-source parallel scrape

`StreamHealthService` enriches streams using:

- HTTP/HTTPS trackers via `TrackerScrapeClient`
- UDP trackers via `UdpScrapeClient` (BEP-48)
- DHT peer discovery via `DhtPeerDiscovery` (BEP-5)

Results are merged by highest seeder count. Lazy DHT bootstrap happens once before fan-out. Cache-first lookup with <30s TTL provides instant results; merged results are persisted under the synthetic key `multi://merged`.

### TrackerScorer

Bayesian Average + EWMA scoring algorithm:

- Bayesian success rate with confidence parameter `C=10` (shrinks low-sample trackers toward the global mean).
- EWMA of latency (`α=0.2`) and peer count.
- Weighted score: success 0.5, latency 0.3, peers 0.2.
- Protocol bonus: UDP +0.05, HTTPS +0.03.
- Backoff penalty: trackers in backoff window score 0.0.
- Score range: [0, 1.05].

### TrackerHealthMonitor

- Active probing with exponential backoff: 1m → 5m → 15m → 1h → 6h → 24h.
- Top-20 trackers probed every 5 min; rest every 30 min.
- Parallel probing with max 10 concurrent.
- `pruneDeadTrackers()` removes trackers dead >24h.
- Uses a placeholder info hash for probing; comment suggests replacing with a perennially-seeded public-domain torrent.

### UdpScrapeClient

- BEP-48 two-step transaction: connect (16-byte request) → scrape (36+ byte request).
- 4-second timeout per transaction.
- Reuses `ScrapeResult` from HTTP client.
- Caches results via `TrackerCacheRepository` (30s TTL).
- Creates sockets per request and destroys them immediately.

### DhtPeerDiscovery

- Wraps `bittorrent_dht` package.
- Bootstraps from 5 well-known DHT nodes.
- Accepts 40-char hex info hash, converts to 20-byte raw string.
- Event-based peer collection via `NewPeerEvent` listener.
- 15-second timeout for peer collection.
- Supports `announce()` to help other nodes discover the app.

### MagnetEnhancer

- Appends tracker URLs as `&tr=` parameters.
- De-duplicates trackers already present in the magnet.
- Fetches from `TrackerList` (public tracker list).
- Max 12 trackers appended by default.
- URL-encodes trackers before appending.

### StreamResolver

- Distinguishes CIDs (IPFS) from BitTorrent info hashes.
- CID validation: v0 (`Qm` prefix, 46 chars) or v1 (`bafy`/`bafk`, 59+ chars).
- Resolves CIDs to public IPFS gateways (ipfs.io, cloudflare-ipfs.com, dweb.link) and local gateway (`127.0.0.1:8080`).
- Extracts web seed (`ws=`) parameter from magnet links.
- Returns `null` for info hashes (cannot resolve via IPFS gateway).

## Recent decisions

- Community providers migrated from `.community_plugins` pack to a compiled-in registry; 6 providers disabled due to site issues.
- Internet Archive community provider removed to avoid ID conflict with the built-in provider.
- Cache-first peer state lookup with fire-and-forget background burst scrape (3s budget) in `ProviderAggregator`.

## Quirks and issues

- `ScraperManager._fetchCachedStreams` returns `null` if any registered provider has no cache entry, preventing partial cache hits from hiding streams from providers registered later.
- `ScraperManager.fetchStreams` does NOT apply DMCA blocklist filtering; that is the caller's responsibility (performed in `ProviderAggregator`).
- NASA provider skips `tt` IDs to avoid masking other providers.
- `DhtPeerDiscovery.requestPeers` returns empty list if DHT is not initialized; caller must call `initialize()` first.
- `TrackerHealthMonitor` uses a placeholder info hash for probing; realistic probing requires a perennially-seeded public-domain torrent.

## SQLite schema

Schema v7 added a `tracker_stats` table:

- `tracker_url`, `protocol`, `success_count`, `total_count`, `ewma_latency`, `ewma_peers`, `last_scrape_at`, `backoff_until`, `consecutive_failures`, `score`
- Index: `idx_tracker_stats_score`

## Test coverage

- `test/scraper/*` — built-in provider tests, scraper manager, DHT peer discovery, magnet enhancer, stream resolver, tracker scorer, tracker health monitor, UDP scrape client.
- `test/scraper/provider_probe_test.dart` — provider probe logic.
- `test/scraper/tracker_scorer_test.dart` — tracker scoring.
- `test/scraper/tracker_health_monitor_test.dart` — health monitor.
- `test/scraper/udp_scrape_client_test.dart` — UDP scrape protocol.
- `test/scraper/known_torrents_provider_test.dart` — known torrents provider.
- `test/scraper/torznab_provider_test.dart` — Torznab provider.
- Sprint 11 networking components may have limited coverage due to reliance on external network services; protocol logic is unit-tested where possible.

## Related

- [[ciel/projects/X-Seed/knowledgebase.md|X-Seed — Knowledgebase]]
- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|Addon / Stremio]]
- [[ciel/projects/X-Seed/subsystems/ui-ux-routing.md|UI / UX / Routing]]
- [[ciel/projects/X-Seed/subsystems/background-services.md|Background Services]]
- [[ciel/projects/X-Seed/subsystems/security-build-ci.md|Security / Build / CI]]
