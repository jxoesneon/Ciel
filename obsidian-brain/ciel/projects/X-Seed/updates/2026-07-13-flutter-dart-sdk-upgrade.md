---
title: X-Seed Flutter/Dart SDK Upgrade to Latest Master
type: project-note
project_note: update
tags: ["project-note","update"]
status: active
created: "2026-07-13T00:00:00Z"
---

# X-Seed Flutter/Dart SDK Upgrade to Latest Master

## Summary
Upgraded the Flutter SDK from stable `3.44.4` (Dart `3.12.2`) to the latest `master` channel `3.46.0-1.0.pre-533` (Dart `3.14.0-10.0.dev`). Updated all mutually compatible dependencies, fixed new analyzer warnings, and verified the full suite and Android builds.

## Changes Made

### Flutter SDK
- Switched from `stable` to `master` channel because the latest stable release still shipped Dart `3.12.2`, which pinned `test_api`/`test`/`test_core` and prevented several package upgrades.
- New SDK: Flutter `3.46.0-1.0.pre-533`, Dart `3.14.0-10.0.dev`, DevTools `2.60.0`.

### Dart Package Upgrades
Direct dependency constraints updated:
- `local_auth` ^2.3.0 → ^3.0.2
- `local_auth_android` ^1.0.56 → ^2.0.9
- `connectivity_plus` ^6.1.0 → ^7.2.0
- `firebase_core` ^3.0.0 → ^4.11.0
- `firebase_crashlytics` ^4.0.0 → ^5.2.4
- `firebase_analytics` ^11.0.0 → ^12.4.3
- `flutter_secure_storage` ^10.2.0 → ^10.3.1

After `flutter pub upgrade`, 11 transitive packages moved to their newest resolvable versions (e.g. `intl` 0.20.3, `meta` 1.19.0, `mockito` 5.7.0, `vector_math` 2.4.0, analyzer/dart_style/test_api/test_core, etc.).

### Android Tooling
- Updated `android/settings.gradle` Kotlin plugin: `2.2.20` → `2.4.0`.
- Attempted AGP `8.11.1` → `9.2.0` and Gradle `8.14` → `9.4.1` with built-in Kotlin, but reverted because the Flutter Gradle plugin in this master revision does not yet support AGP 9's new DSL interface.
- Left AGP at `8.11.1` and Gradle at `8.14`; build succeeds with deprecation warnings.

### Code Fixes for New SDK
- Fixed two new `unawaited_return_in_try_block` warnings:
  - `lib/src/features/bridge/subtitle_proxy_service.dart:109`
  - `lib/src/features/popularity/sqlite_content_popularity_service.dart:331`
- Updated `biometric_gate.dart` and its tests for the `local_auth` 3.x API signature change.

## Verification

```powershell
cd C:/Users/josee/X-Seed/x_seed
flutter analyze --fatal-infos       # No issues found
flutter test                        # 2178 passed, 0 failed
flutter test --coverage             # 81.62% line coverage
flutter build apk --debug --flavor play   # Built app-play-debug.apk
flutter build apk --debug --flavor full   # Built app-full-debug.apk
```

## Remaining Blockers
- **AGP 9 / built-in Kotlin**: Cannot migrate until the Flutter Gradle plugin supports AGP 9's new DSL.
- **10 outdated transitive packages**: blocked by the external `dart_ipfs` path dependency (`xml`, `dart_udx`, `package_config`, `qr`) or by Flutter SDK pins (`test`, `test_api`, `test_core`, `_fe_analyzer_shared`, `analyzer`, `flutter_secure_storage_darwin`).
- **Java 8 obsolete warning**: emitted by a plugin/subproject; non-fatal.
- **KGP deprecation warning**: still emitted by `app_settings`, `firebase_analytics`, `jailbreak_root_detection`, `workmanager_android`; no compatible plugin releases exist.

## Files of Note
- `C:/Users/josee/X-Seed/x_seed/AGENTS.md` — updated baselines and known issues.
- `C:/Users/josee/X-Seed/x_seed/pubspec.yaml` and `pubspec.lock`.
- `C:/Users/josee/X-Seed/x_seed/android/settings.gradle`.
