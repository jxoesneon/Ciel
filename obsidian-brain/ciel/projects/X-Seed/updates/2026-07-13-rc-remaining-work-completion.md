---
title: X-Seed v1.0.0-RC-1 Remaining Work — Completion Report
type: project-note
project_note: update
tags: ["project-note","update"]
status: active
created: "2026-07-13T00:00:00Z"
---

# X-Seed v1.0.0-RC-1 Remaining Work — Completion Report

## Goal
Implement all remaining previously-accepted non-blockers and Sprint 8/9 work so that v1.0.0-RC-1 is comprehensively complete.

## What Was Implemented

### Dependency / Plugin Maintenance
- Upgraded direct dependencies where compatible:
  - `local_auth` ^2.3.0 → ^3.0.2
  - `local_auth_android` ^1.0.56 → ^2.0.9
  - `connectivity_plus` ^6.1.0 → ^7.2.0
  - `firebase_core` ^3.0.0 → ^4.11.0
  - `firebase_crashlytics` ^4.0.0 → ^5.2.4
  - `firebase_analytics` ^11.0.0 → ^12.4.3
- Updated `biometric_gate.dart` and its tests for the `local_auth` 3.x API.
- 16 transitive packages remain on older versions because they require a newer Flutter/Dart SDK than the current environment.
- **KGP deprecation warning remains** for `app_settings`, `firebase_analytics`, `jailbreak_root_detection`, `workmanager_android` because no built-in-Kotlin compatible versions are published yet.

### Sprint 8 Items
- **HiveService custom adapters**: Determined no custom adapters are needed (all stored types are primitives or JSON-serializable maps); removed the TODO and documented the rationale.
- **OpenSubtitles subtitle download**: Replaced the `subtitleDownloadPlaceholder` with a real download flow wired through `subtitle_section.dart`, complete with loading indicator and localized `subtitleDownloading` string.

### Sprint 9 Items
- **Identity Key management UI**: Implemented `identity_key_section.dart` in Settings with key status, export/import, and seed-phrase backup actions, all behind `BiometricGate`.
- **BIP39 seed-phrase backup/recovery**: Added `exportSeedPhrase()` and `importSeedPhrase()` to `KeystoreService`, using `bip39_plus: ^1.1.1` (compatible with the existing `pointycastle` constraint).

### Previously Accepted Non-Blockers
- **KeystoreService dual-storage**: Removed the raw base64 fallback so Tier 1/2 encrypted keys are no longer duplicated in raw form.
- **Addon self-signed certificate**: Implemented `AddonCertManager` to generate and persist a per-install RSA self-signed certificate in `FlutterSecureStorage`; HTTPS listener now uses the generated cert, with the bundled asset kept only as an emergency fallback.
- **SwarmService per-CID health**: Removed the TODO and implemented `refreshHealth()` using `IPFSNode.findProviders(cid)` as the per-CID peer estimate.

## Final Verification

```powershell
cd C:/Users/josee/X-Seed/x_seed
flutter analyze --fatal-infos       # No issues found
flutter test                        # 2178 passed, 0 failed, 0 skipped
flutter test --coverage             # 81.62% line coverage
flutter build apk --debug --flavor play   # Built app-play-debug.apk
flutter build apk --debug --flavor full   # Built app-full-debug.apk
```

## What Could Not Be Fully Resolved
- **Kotlin Gradle Plugin deprecation**: No compatible plugin releases exist for `app_settings`, `jailbreak_root_detection`, and `workmanager_android`. The app still builds, but future Flutter versions may fail. This is tracked as external maintenance debt.
- **16 outdated transitive packages**: Blocked by the installed Flutter/Dart SDK ceiling. Upgrading the Flutter SDK itself was out of scope.

## Files of Note
- `C:/Users/josee/X-Seed/x_seed/AGENTS.md` — updated baselines and RC audit section.
- `C:/Users/josee/X-Seed/x_seed/.scratch/rc_remaining_work_loop.md` — detailed loop tracker.
