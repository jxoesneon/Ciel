---
title: X-Seed catalog search as URL path segment
type: decision
project: X-Seed
tags: [decision, x-seed, addon, catalog, search, stremio]
status: accepted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Decision: X-Seed catalog search as URL path segment

## Status

Accepted

## Context

Stremio sends catalog extras (such as search queries) as a URL path segment rather than a query string. The original addon server only registered the route:

```
/catalog/<type>/<id>.json
```

Requests like `/catalog/movie/xseed_popular/search=QUERY.json` were not matched, causing 404 errors and preventing X-Seed content from appearing in Stremio search results.

## Decision

Add a new route to the addon server:

```
/catalog/<type>/<id>/<extra>.json
```

The handler parses the `extra` segment to extract search queries, genre filters, and skip offsets. For example, `search=QUERY` is parsed and passed to the catalog handler, which delegates to the scraper manager.

Implementation in `x_seed/lib/src/features/addon/addon_server.dart` (route registration and `_catalogHandlerWithExtra`).

## Consequences

- **Positive**: Stremio search queries now reach the X-Seed addon and return results from the scraper manager.
- **Positive**: Backward-compatible with the original `/catalog/<type>/<id>.json` route.
- **Trade-off**: The `extra` segment may contain URL-encoded characters, so decoding must be applied before parsing.
- **Trade-off**: The spec (`ADDON_API_SPEC.md`) currently documents search as a query parameter; it should be updated to reflect the path-segment reality.

## Related

- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|X-Seed — Addon / Stremio]]
- `x_seed/lib/src/features/addon/addon_server.dart`
- `x_seed/test/addon/addon_server_endpoints_test.dart`
- `docs/specs/ADDON_API_SPEC.md`
