---
title: X-Seed — Addon / Stremio
project_note: subsystem
type: project-note
project: X-Seed
tags: [subsystem, x-seed, addon, stremio, deep-link]
status: active
created: "2026-07-12T07:44:58.462Z"
---

# X-Seed — Addon / Stremio

Local Stremio Addon SDK v3 server and all integration points with the Stremio client.

## Summary

X-Seed runs a local `shelf` HTTP addon server bound to `127.0.0.1:7979` (HTTP) with an optional HTTPS listener on `7987`. The addon exposes manifest, catalog, meta, stream, and subtitles endpoints so Stremio can consume X-Seed as a local content source. A Fly.io redirect proxy (`x-seed-forge.fly.dev`) serves the manifest over HTTPS and redirects resource requests back to the local HTTP server, enabling native `stremio://` addon installation on Android despite Stremio's rustls TLS validation.

## Key files

| File | Purpose |
|------|---------|
| `x_seed/lib/src/features/addon/addon_server.dart` | Main HTTP server with shelf_router, middleware, and endpoint handlers |
| `x_seed/lib/src/features/addon/id_parser.dart` | StreamId parsing for IMDb, series, magnet/btih, and raw IDs |
| `x_seed/lib/src/features/addon/meta_service.dart` | Metadata orchestration (cache → Cinemeta fallback) |
| `x_seed/lib/src/features/addon/provider_aggregator.dart` | Stream fetching, deduplication, DMCA filtering, normalization |
| `x_seed/lib/src/features/addon/stream_models.dart` | Stremio-compatible stream response models |
| `x_seed/lib/src/services/ui/external_player_launcher.dart` | Deep link construction for addon install, detail, and player |
| `x_seed/lib/src/features/ui/detail/stream_action_sheet.dart` | UI action sheet for "Open in Stremio" flows |
| `x_seed/lib/src/features/ui/detail/stremio_open_dialog.dart` | Dialog handling install vs. open actions |
| `forge/server.dart` | Fly.io redirect proxy |
| `docs/specs/ADDON_API_SPEC.md` | API specification |
| `docs/adr/001-local-addon-host.md` | ADR for local addon hosting |

## Endpoints

The addon server registers the following routes (all with OPTIONS preflight handlers):

- `GET /manifest.json` — addon manifest
- `GET /configure` — HTML redirect page → `xseed://settings`
- `GET /catalog/<type>/<id>.json` — catalog without extras
- `GET /catalog/<type>/<id>/<extra>.json` — catalog with extras (e.g., `search=QUERY`)
- `GET /meta/<type>/<id>.json` — metadata
- `GET /stream/<type>/<id>.json` — streams
- `GET /subtitles/<type>/<id>.json` — subtitles
- `GET /subtitle_download/<id>` — subtitle file download
- `GET /no-sources` — consolation endpoint

## Middleware pipeline

Applied in order:

1. `_debugLoggingMiddleware` — request/response logging
2. `_proxyHeaderStripMiddleware` — removes `X-Forwarded-*` headers
3. `_loopbackOnlyMiddleware` — rejects non-loopback requests
4. `_rateLimitMiddleware` — 100 req/min per IP, 20 req/min per endpoint
5. `_corsMiddleware` — CORS headers
6. `_requestCountingMiddleware` — request metrics
7. `_addonIdleMiddleware` — auto-shutdown after inactivity

## ID parsing and URL decoding

`StreamIdParser.parse()` handles:

- `tt1234567` → movie IMDb ID
- `tt1234567:1:5` → series IMDb ID with season/episode
- `magnet:?xt=urn:btih:HASH` → `StreamId.btih(hash)`
- `btih:HASH` → `StreamId.btih(hash)`
- any other format → `StreamId.raw(id)`

The addon server URL-decodes IDs before parsing (`Uri.decodeComponent(id)`). This is critical for magnet URLs containing `?`, `:`, and `=` characters when they appear as URL path segments.

## Stremio deep links

### Player deep link (`launchStremioPlayer`)

For `btih`-based content, the working deep link format is:

```
stremio:///player/{encodedStream}
```

where `encodedStream` is constructed as:

```dart
final streamJson = jsonEncode({
  'infoHash': infoHash,
  'sources': sources.map((s) => s.startsWith('tracker:') ? s : 'tracker:$s').toList(),
  'name': title,
});
final compressed = ZLibEncoder(level: 0).convert(utf8.encode(streamJson));
final encoded = base64Encode(compressed);
final urlEncoded = Uri.encodeComponent(encoded);
return 'stremio:///player/$urlEncoded';
```

Sources are prefixed with `tracker:` or `dht:` as required by Stremio.

### Detail deep link (`launchStremioDetail`)

- IMDb/Kitsu IDs: `stremio:///detail/{type}/{id}/{videoId}` (videoId URL-encoded)
- Magnet/raw IDs: copies title to clipboard, opens `stremio:///search?search={encodedTitle}` because Stremio Android only queries Cinemeta for detail links, not installed addons.

### Addon install deep link (`launchStremioAddon`)

Two-button strategy:

1. **Primary**: `stremio://x-seed-forge.fly.dev/manifest.json` — stremio-web converts to `https://x-seed-forge.fly.dev/manifest.json`; rustls accepts the Fly.io Let's Encrypt cert. The forge then redirects resources to the local HTTP server.
2. **Fallback**: open `https://web.stremio.com/#/addons?addon={encodedManifestUrl}` in Chrome; Chrome allows mixed content for `127.0.0.1`.

The manifest URL is always copied to the clipboard as a backup.

## Forge proxy

`forge/server.dart` is a stateless Dart shelf app deployed on Fly.io:

- Serves `/manifest.json` over HTTPS with a valid Let's Encrypt certificate.
- Redirects `/catalog/*`, `/meta/*`, `/stream/*`, `/subtitles/*`, `/subtitle_download/*`, and `/no-sources` to `http://127.0.0.1:{port}/{path}` with HTTP 302.
- Reads `?port=N` query parameter and preserves it across redirects (stremio-core preserves query parameters).
- Serves `/configure` as an HTML page that redirects to `xseed://settings`.
- The manifest declares `idPrefixes: ['tt', 'kitsu', 'magnet', 'btih']`.

## Stream aggregation

`ProviderAggregator.fetchStreams()`:

1. Direct btih → magnet construction
2. Otherwise delegate to `ScraperManager.fetchStreams()`
3. Normalize titles
4. Extract quality/size from original titles before normalization
5. Deduplicate streams
6. Filter non-video streams defensively
7. Sort by seeders then quality
8. Apply DMCA blocklist via `BlocklistService`
9. Log DMCA blocks via `SecurityLogService`

## Recent decisions

- [[ciel/kg/decisions/xseed-stremio-player-deep-link.md|Stremio player deep link format]]
- [[ciel/kg/decisions/xseed-magnet-btih-id-routing.md|Magnet / btih ID routing]]
- [[ciel/kg/decisions/xseed-catalog-search-path-segment.md|Catalog search as path segment]]
- [[ciel/kg/decisions/xseed-forge-redirect-proxy.md|X-Seed Forge redirect proxy]]

## Quirks and issues

- Self-signed HTTPS on port 7987 is rejected by Stremio Android because rustls uses webpki-roots (Mozilla CA bundle) and does not accept self-signed certs. The Forge proxy is the working solution; the HTTPS listener is kept as future-proofing.
- Stremio Android detail deep links only query Cinemeta for meta, not installed addons. For non-IMDb IDs, the app falls back to title search.
- Previous default port `11470` collided with `stremio_server`; changed to `7979` with fallback range `7980–7988`.
- Trackerless magnets get a default public tracker and DHT bootstrap node injected.

## Test coverage

- `test/addon/addon_server_endpoints_test.dart` — meta, stream, catalog, manifest, middleware, URL decoding for magnet IDs.
- `test/services/ui/external_player_launcher_test.dart` — basic launch, addon install, detail deep links, player deep link encoding.
- `test/addon/id_parser_test.dart` — ID parsing for all supported formats.
- `test/addon/provider_aggregator_test.dart` — stream aggregation, DMCA filtering, deduplication.

## Related

- [[ciel/projects/X-Seed/knowledgebase.md|X-Seed — Knowledgebase]]
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|Providers / Scraper]]
- [[ciel/projects/X-Seed/subsystems/ui-ux-routing.md|UI / UX / Routing]]
- [[ciel/projects/X-Seed/subsystems/background-services.md|Background Services]]
- [[ciel/projects/X-Seed/subsystems/security-build-ci.md|Security / Build / CI]]
