---
title: X-Seed Forge HTTPS-to-HTTP redirect proxy
type: decision
project: X-Seed
tags: [decision, x-seed, forge, stremio, proxy, https, flyio]
status: accepted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Decision: X-Seed Forge HTTPS-to-HTTP redirect proxy

## Status

Accepted

## Context

Stremio's Android app uses `rustls` for TLS, which only trusts the Mozilla webpki-roots CA bundle. The X-Seed addon server is HTTP-only on the device loopback (`127.0.0.1:7979`). When a `stremio://` deep link is opened, stremio-core's `deserialize_transport_url` hardcodes conversion to `https://`, causing the native Stremio app to fail fetching the manifest from the local HTTP server. A self-signed HTTPS listener on `127.0.0.1:7987` was also tested and rejected by rustls because it does not accept self-signed certificates.

Empirically confirmed failures:

- `stremio://127.0.0.1:7979/manifest.json` → converted to `https://127.0.0.1:7979/manifest.json` → fails (HTTP server, HTTPS request).
- `stremio://127.0.0.1:7987/manifest.json` → TLS handshake fails (self-signed cert rejected by rustls/webpki-roots).
- `stremio:///addons?addon=...` → silently dropped by the Android Stremio Navigator.

## Decision

Deploy a thin redirect proxy on Fly.io at `https://x-seed-forge.fly.dev`:

1. Serve `/manifest.json` over HTTPS using Fly.io's Let's Encrypt certificate (trusted by rustls/webpki-roots).
2. Redirect all resource paths (`/catalog/*`, `/meta/*`, `/stream/*`, `/subtitles/*`, `/subtitle_download/*`, `/no-sources`) via HTTP 302 to `http://127.0.0.1:{port}/{path}`.
3. Preserve the `?port=N` query parameter across redirects so the proxy can target non-default addon ports.
4. Serve `/configure` as an HTML page that redirects to `xseed://settings` so Stremio's "Configure" button opens X-Seed settings.
5. Use `stremio://x-seed-forge.fly.dev/manifest.json` as the primary native install deep link; fallback to Stremio Web in Chrome.

`reqwest` (used by Stremio's fetch client) follows 302 redirects by default, including HTTPS→HTTP cross-protocol redirects.

## Consequences

- **Positive**: Native Stremio Android addon installation works without self-signed certificates.
- **Positive**: The proxy is stateless and does no compute; all addon data still flows directly from the local device to Stremio.
- **Positive**: Stremio Web fallback (`https://web.stremio.com/#/addons?addon=...`) remains available for cases where the native deep link fails.
- **Trade-off**: Requires a public HTTPS endpoint (Fly.io) and ongoing deployment/maintenance.
- **Trade-off**: Adds network latency only for the manifest fetch; all subsequent resource requests are local after the 302 redirect.
- **Trade-off**: Privacy-conscious users may prefer the manual "Stremio Web" install path over the Forge proxy.

## Related

- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|X-Seed — Addon / Stremio]]
- `forge/server.dart`
- `forge/fly.toml`
- `forge/Dockerfile`
- `x_seed/lib/src/services/ui/external_player_launcher.dart`
- `x_seed/AGENTS.md` ("Stremio addon install deep link" and "X-Seed Forge" sections)
