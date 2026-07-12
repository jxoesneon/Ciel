---
title: Dart/Flutter Wireless Debugging
type: concept
project: X-Seed
tags: [concept, flutter, dart, adb, wireless-debugging, mobile-testing, stremio]
created: 2026-07-11
status: active
---

# Dart/Flutter Wireless Debugging

A playbook for connecting to physical Android/iOS devices over Wi-Fi and running Flutter apps, tests, and verification flows without a USB cable.

## Core idea

Modern Android devices (Android 11+) advertise ADB over TLS via mDNS (`_adb-tls-connect._tcp`) after a one-time pairing. Flutter tooling can target these devices by serial once ADB sees them. When accessibility tooling fails on Flutter screens (common on `CustomScrollView`/Surface rendering), fall back to ADB screenshots and coordinate taps, and prefer integration tests or deep links over brittle multi-step UI automation.

## Key techniques

- **ADB TLS/mDNS discovery**: `adb mdns services` → `adb pair <ip>:<pair_port>` → `adb connect <ip>:<connect_port>`.
- **Flavored Flutter commands**: Always pass `--flavor` and `--target` when the project uses product flavors; otherwise Gradle may not produce an APK.
- **Screenshot-first observation**: `adb shell screencap -p` works even when `uiautomator` returns a null root node.
- **Direct intent/deep-link shortcuts**: Launch specific screens with `adb shell am start -a VIEW -d "xseed://..."` to bypass fragile coordinate navigation.

## Tooling matrix

| Task | Preferred tool | Fallback |
|------|----------------|----------|
| Device discovery | `flutter devices` | `adb devices -l`, `adb mdns services` |
| Install/run app | `flutter run --device-id ... --flavor ...` | `adb install -r app.apk` then `adb shell am start` |
| Take screenshot | `flutter screenshot` | `adb shell screencap -p` |
| UI automation | Integration test (`flutter test integration_test`) | ADB coordinate taps based on screenshot |
| Drive external apps | `ExternalPlayerLauncher` + `url_launcher` | `adb shell am start -d <intent>` |

## Known Flutter-specific gotchas

- Android UIAutomator often cannot see Flutter widget trees; expect `null root node` from `uiautomator dump`.
- `mobile-mcp` element listing inherits the same UIAutomator limitation for Flutter apps.
- High-DPI devices (e.g., 1440x3168 @ 640dpi) make coordinate-based tapping error-prone if density scaling is not accounted for.
- Deep links delivered to a running activity may be ignored if the route guard or `go_router` redirect does not re-evaluate the intent.
- A splash screen that never dismisses can indicate that `main()` is awaiting a foreground service or other blocking native call. On Android 14+ the `dataSync` foreground-service start budget can be exhausted, causing `runApp()` to never execute. A debug-only workaround is to wrap the service start in try-catch; the real fix belongs in the service lifecycle.

## Artifact hygiene

Debugging a physical device creates transient artifacts (screenshots, logs, APK copies). Keep them out of the codebase in `.scratch/wireless-debug/<yyyy-mm-dd>/` or the system temp directory. This is a technical workflow concern, not content creation. Non-technical device data (personal media, messages, etc.) is never within scope.

## Self-improvement trigger

The skill treats repeat failures as a mandatory evolution signal. If the same failure occurs **3 times in a row** across invocations, stop retrying and run a self-improvement pass: update the troubleshooting table, add a guard step that switches to a less brittle path, and never retry the same broken approach again.

## References

- Skill implementation: `.devin/skills/dart-wireless-debug/SKILL.md`
- Applied during:
  - [[ciel/diary/2026-07-11-xseed-stremio-phone-verification.md|X-Seed Stremio phone verification]]
  - [[ciel/diary/2026-07-12-detail-episode-filtering-wiring.md|X-Seed detail episode filtering wiring]]
- Related project: [[ciel/projects/X-Seed/X-Seed.md|X-Seed]]
