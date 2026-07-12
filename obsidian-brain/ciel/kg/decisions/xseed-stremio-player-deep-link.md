---
title: X-Seed Stremio player deep link format
type: decision
project: X-Seed
tags: [decision, x-seed, stremio, deep-link, btih]
status: accepted
date: 2026-07-11
created: 2026-07-11
decision_date: 2026-07-11
---

# Decision: X-Seed Stremio player deep link format

## Status

Accepted

## Context

X-Seed needs to open torrent content directly in the Stremio Android app. Several Stremio deep link formats were tried and found broken for `btih` content:

- `stremio:///detail/{type}/{id}/...` — Stremio only queries Cinemeta for detail links, not installed addons. X-Seed content with `btih:` or `magnet:` IDs never resolves.
- `stremio:///search?search=...` — does not auto-execute the search; requires additional user interaction.
- Raw `magnet:` URLs — cause Stremio to get stuck in a "parsing magnet link" state.

The working solution is Stremio's player deep link format, which bypasses the detail/search flows and opens the player directly with a zlib-compressed, base64-encoded stream descriptor.

## Decision

For `btih`-based content, use the `stremio:///player/{encodedStream}` deep link format.

The encoded stream is constructed as:

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

Implementation lives in `x_seed/lib/src/services/ui/external_player_launcher.dart` (`launchStremioPlayer`).

## Consequences

- **Positive**: Directly opens Stremio's player for torrent content without relying on Cinemeta or search.
- **Positive**: Works for `btih:` and `magnet:?xt=urn:btih:HASH` content.
- **Trade-off**: Requires the caller to construct the stream descriptor; sources must be prefixed with `tracker:` or `dht:` as required by Stremio.
- **Trade-off**: Does not provide a detail page; only the player is opened.

## Related

- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|X-Seed — Addon / Stremio]]
- [[ciel/projects/X-Seed/goals/full-mine-obsidian-brain-2026-07-11.md|Full mine goal]]
- `x_seed/lib/src/services/ui/external_player_launcher.dart`
- `x_seed/lib/src/features/ui/detail/stremio_open_dialog.dart`
