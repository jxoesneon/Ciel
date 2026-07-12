---
title: X-Seed — Background Services
project_note: subsystem
type: project-note
project: X-Seed
tags: [subsystem, x-seed, background-service, workmanager, foreground-service]
status: active
created: "2026-07-12T07:44:58.463Z"
---

# X-Seed — Background Services

Android foreground service, WorkManager periodic tasks, and isolate communication that keep the addon server and IPFS/libp2p node alive.

## Summary

The Background Services subsystem runs the addon HTTP server and IPFS/libp2p node inside a Flutter foreground service via `flutter_background_service`. It uses `WorkManager` for periodic resurrection, tracker refresh, and pre-cache tasks, and communicates status from the background isolate to the main isolate via `service.invoke()`. The subsystem also includes battery optimization guidance, rotating on-disk logs, and flavor-specific bootstrapping for `play` and `full` builds.

## Key files

| File | Purpose |
|------|---------|
| `x_seed/lib/src/services/background_service.dart` | Main foreground service implementation (798 lines) |
| `x_seed/lib/src/services/background_service_interface.dart` | Interface contract and state enum |
| `x_seed/lib/src/services/background/fake_background_service.dart` | Test double for UI testing |
| `x_seed/lib/src/services/background/resurrection_service.dart` | WorkManager resurrection task (15-min interval) |
| `x_seed/lib/src/services/background/battery_optimization_service.dart` | OEM-specific battery optimization guidance |
| `x_seed/lib/src/services/background/log_service.dart` | Rotating on-disk logging with real-time stream |
| `x_seed/lib/src/services/background/precache_service.dart` | WorkManager pre-cache task (6h interval) |
| `x_seed/lib/src/services/background/tracker_refresh_service.dart` | WorkManager tracker refresh task (6h interval) |
| `x_seed/lib/src/services/background/watchlist_precache.dart` | Startup watchlist meta pre-cache |
| `x_seed/lib/src/features/bridge/background_status_provider.dart` | Real status provider for UI |
| `x_seed/lib/main_common.dart` | Shared bootstrap for both flavors |
| `x_seed/lib/main.dart` | Full flavor entry point |
| `x_seed/lib/main_play.dart` | Play flavor entry point |
| `docs/specs/BACKGROUND_SERVICE_SPEC.md` | Specification |

## Service lifecycle

- **Entry point**: `@pragma('vm:entry-point')` annotated `_onStart` method.
- **Foreground service type**: `dataSync`.
- **Notification channel**: `xseed_foreground` with ID 100.
- **Addon server**: Started on port 7979 (fallback 7980–7988) in the background isolate.
- **IPFS node**: Co-located in the same isolate.
- Lifecycle methods: `initialize()`, `start()`, `stop()`, `resume()`.

## Main isolate status updates

The main isolate cannot directly read static fields from the background isolate. The service uses mirror fields:

- `_mainIsolateAddonOnline`, `_mainIsolateAddonPort`, `_mainIsolateAddonRequestCount`
- Background isolate sends `addonStatus` events every 2 seconds via `service.invoke('addonStatus', {online, port, requests})`.
- Main isolate listener updates mirror fields in `BackgroundService.initialize()`.
- UI reads `isAddonServerRunning`, `addonPort`, `addonRequestCount` from the main isolate.

State changes are broadcast via:

- `_stateController` → `BackgroundServiceState` (idle, running, paused, stopping)
- `_nodeStateController` → `IpfsNodeState` for IPFS node metrics

## Resurrection and boot receiver

- **ResurrectionService**: WorkManager task named `xseed-resurrection`, 15-minute interval, unmetered network constraint.
  - 3 consecutive failures → notification.
  - 5 consecutive failures → dialog on next app launch.
  - Respects `BackgroundServicePreferences.manualStop` flag.
- **BootReceiverManager**: Registered in `main_common.dart`; auto-starts service on boot if "Start on boot" is enabled.

## Battery optimization

- `BatteryOptimizationService` detects Samsung, Xiaomi, Huawei, OnePlus, and stock Android.
- Uses custom MethodChannel `com.jxoesneon.x_seed/battery` to avoid `permission_handler` issues on Android 16.
- Returns OEM-specific instructions for disabling battery restrictions.

## Periodic tasks

| Task | Interval | Constraints | Trigger |
|------|----------|-------------|---------|
| Pre-cache popular | 6h | unmetered + charging | `PreCacheService` invokes `preCachePopular` event |
| Tracker refresh | 6h | network connected | `TrackerRefreshService` invokes `refreshTrackersPeriodic` event |
| Resurrection | 15m | unmetered network | `ResurrectionService` checks and restarts service |

## Precache and watchlist

- **PreCacheService**: invokes `preCachePopular` on the background isolate; silently skips if service is not running.
- **WatchlistPreCache**: triggered at app startup in `main_common.dart`; 10 concurrent requests max; individual failures don't abort the batch; fire-and-forget.

## Logging service

- **File location**: `<appDocuments>/logs/service.log`.
- **Rotation**: 5 generations, 2MB max size.
- **Format**: `yyyy-MM-ddTHH:mm:ss.mmm [LEVEL] (category) message`.
- **Levels**: debug, info, warn, error.
- **Real-time stream**: `entryStream` broadcasts `LogEntry` objects.
- **Operations**: `tail()`, `clearLogs()`, `exportLogs()`.
- **Export**: concatenates rotated files oldest-first, saves to temp directory.

## Flavor differences

- **Full flavor (`main.dart`)**: `NoOpMonitoringService`, GitHub update check enabled, no Firebase.
- **Play flavor (`main_play.dart`)**: `FirebaseMonitoringService` (Crashlytics + Analytics), GitHub update check disabled, Firebase initialized.
- **Shared bootstrap (`main_common.dart`)**: wires both flavors with flavor-injected monitoring service; WorkManager tasks registered in both.

## Recent changes

- Background isolate now sends `addonStatus` events every 2 seconds to keep the main isolate in sync.
- Tracker optimization integration added (Sprint 11): tracker refresh service with health monitoring.
- Android 15 6-hour foreground service limit documented; pre-emptive notification at 5h45m and graceful auto-restart via resurrection.
- Custom MethodChannel for battery optimization to avoid `permission_handler` issues on Android 16.

## Quirks and issues

- Isolate communication requires mirror fields because static fields are isolate-specific in Dart.
- Pre-cache and tracker refresh silently skip if the service is not running.
- Notification icon must use `ic_notification` (drawable), not `ic_launcher` (mipmap).
- Android 15 limits foreground services to 6 hours; resurrection is the workaround.
- OEM battery optimization variability requires device-specific instructions.

## Test coverage

- `test/services/background_service_test.dart` — background service lifecycle and status events.
- `test/services/background/*` — resurrection, precache, tracker refresh, log service.
- `test/bridge/background_status_provider_test.dart` — status provider.
- `FakeBackgroundService` provides a testable in-memory implementation for UI tests.
- Full integration tests for the resurrection flow require emulator verification (not yet implemented).

## Related

- [[ciel/projects/X-Seed/knowledgebase.md|X-Seed — Knowledgebase]]
- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|Addon / Stremio]]
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|Providers / Scraper]]
- [[ciel/projects/X-Seed/subsystems/ui-ux-routing.md|UI / UX / Routing]]
- [[ciel/projects/X-Seed/subsystems/security-build-ci.md|Security / Build / CI]]
