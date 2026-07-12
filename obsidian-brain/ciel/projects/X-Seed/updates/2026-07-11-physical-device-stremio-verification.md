---
title: "X-Seed: Physical-device Stremio stream launch verification"
project_note: update
type: project-note
project: X-Seed
tags: [update, verification, stremio, phone, adb, wireless-debugging]
date: "2026-07-11T01:30:00.000Z"
status: active
created: "2026-07-11T00:00:00Z"
---

# Physical-device Stremio stream launch verification

## Goal

Verify on a real Android phone that launching a stream from X-Seed opens and plays in the Stremio app.

## Device under test

- Model: OnePlus CPH2583
- Android version: 16 (API 36)
- Connection: Wireless ADB TLS on `192.168.100.149:38485` (discovered via mDNS `_adb-tls-connect._tcp`)
- Apps installed: `com.jxoesneon.x_seed` (X-Seed) and `com.stremio.one` (Stremio)

## What was tested

1. **APK install**: Built and installed `app-arm64-v8a-full-debug.apk` from the current working tree.
2. **App launch**: X-Seed launched successfully; Search tab loaded.
3. **Search**: Searched for "NASA" and received results from public-domain providers.
4. **Detail page**: Opened "NSA - Naša Srpska Arhiva" (first result) and confirmed streams load as `Direct • nasa` / `Direct • public_domain` chips.
5. **Stremio player deep link**: Directly launched `stremio:///player/{encodedStream}` with a Big Buck Bunny-style info hash. Stremio opened and entered the player loading state.

## Blockers / partial results

- **UI chip taps did not open the action sheet**: `ActionChip` taps via ADB coordinates did not trigger `StreamActionSheet`. Root cause not isolated; suspected coordinate mismatch on high-DPI screen or scroll hit-test absorption.
- **Deep-link search refresh**: Sending `xseed://search?query=...` to the already-running X-Seed activity did not refresh the search query; the app remained on the previous search.
- **UIAutomator inaccessible**: `mobile-mcp` element listing and `uiautomator dump` both returned null root node on Flutter screens, so screenshots + coordinate taps were required.
- **Integration test build**: `flutter test integration_test/...` without `--flavor` failed to produce an APK; adding `--flavor full` resolved the build but the run was interrupted before completion.

## Verdict

- **Stremio deep link handoff**: CONFIRMED — the `stremio:///player/{encodedStream}` format opens Stremio and starts the player on the phone.
- **X-Seed launcher correctness**: CONFIRMED at unit-test level — `external_player_launcher_test.dart` passes and exercises the same `launchStremioPlayer` encoding path.
- **Full tap-to-play UI flow**: NOT fully verified end-to-end due to Flutter UI automation limitations on this device. A follow-up with a working integration test or manual tap is recommended before release.

## Artifacts

- Devin skill: `.devin/skills/dart-wireless-debug/SKILL.md`
- Ciel skill: `~/.ciel/skills/dart-wireless-debug/` (with `SKILL.md` and `CHANGELOG.md`)
- Ciel skill bundle: `~/.ciel/archive/dart-wireless-debug.skill` + `.sha256`
- Concept note: [[ciel/kg/concepts/dart-wireless-debugging.md|Dart/Flutter Wireless Debugging]]
- Verification screenshots moved to scratch folder: `.scratch/wireless-debug/2026-07-11/`
  - Policy: technical debugging artifacts go to `.scratch/<domain>/<date>/`, never into source-controlled paths. Non-technical device content is out of scope.

## Skill-evolution notes

- The `ActionChip` ADB coordinate-tap failure is now classified as a **repeat-risk pattern** in the skill. The hard rule is: if the same failure repeats 3 times in a row, stop retrying and evolve the skill.
- The skill now mandates switching to integration tests or deep links after two failed coordinate-tap attempts, rather than brittle retries.

## Recommended next steps

1. Add a focused integration test in `integration_test/stremio_stream_e2e_test.dart` that calls `ExternalPlayerLauncher.launchStremioPlayer` with a known legal torrent and asserts it returns `true` on a physical device.
2. Consider adding a debug/test-only `xseed://stremio-player?infoHash=...` deep-link host to exercise the launcher without navigating the UI.
3. Investigate why `ActionChip` taps via ADB coordinates are unreliable on high-DPI OnePlus devices.
