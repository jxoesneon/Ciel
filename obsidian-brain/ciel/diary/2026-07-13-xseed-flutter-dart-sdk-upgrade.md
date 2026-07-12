---
title: X-Seed Flutter/Dart SDK Upgrade — Session Summary
date: 2026-07-13
project: X-Seed
type: diary
tags: [diary, session, x-seed, flutter, dart]
status: active
created: "2026-07-13T00:00:00Z"
---

# X-Seed Flutter/Dart SDK Upgrade — Session Summary

## What I Did
Continued from the RC remaining-work loop. The user asked to upgrade Flutter and Dart SDK to the absolute latest and update all packages, fixing any issues along the way.

### SDK Upgrade
- Upgraded Flutter from stable `3.44.4` (Dart `3.12.2`) to `master` channel `3.46.0-1.0.pre-533` (Dart `3.14.0-10.0.dev`).
- Stable did not provide a newer Dart SDK, so master was required to unblock package upgrades.

### Package Upgrades
- Ran `flutter pub upgrade --major-versions`.
- Bumped direct dependencies: `local_auth`/`local_auth_android`, `connectivity_plus`, Firebase trio, `flutter_secure_storage`.
- 11 transitive packages upgraded; 10 remain blocked by `dart_ipfs` or Flutter SDK pins.

### Tooling / Build Fixes
- Updated Kotlin plugin to `2.4.0`.
- Attempted AGP 9 + Gradle 9.4.1 + built-in Kotlin migration; reverted after the Flutter Gradle plugin failed on AGP 9's new DSL.
- Fixed two new `unawaited_return_in_try_block` analyzer warnings from the master SDK.
- Updated `biometric_gate.dart` for `local_auth` 3.x API changes.

## Verification
- `flutter analyze --fatal-infos`: clean
- `flutter test`: 2178 passed
- `flutter test --coverage`: 81.62%
- `flutter build apk --debug --flavor play`: success
- `flutter build apk --debug --flavor full`: success

## Blockers / Remaining
- AGP 9 migration blocked by Flutter Gradle plugin support.
- 10 transitive outdated packages blocked externally.
- KGP deprecation and Java 8 obsolete warnings remain non-fatal.

## Files Updated
- `C:/Users/josee/X-Seed/x_seed/AGENTS.md`
- `C:/Users/josee/X-Seed/x_seed/pubspec.yaml`
- `C:/Users/josee/X-Seed/x_seed/pubspec.lock`
- `C:/Users/josee/X-Seed/x_seed/android/settings.gradle`
- `C:/Users/josee/Ciel/obsidian-brain/ciel/projects/X-Seed/updates/2026-07-13-flutter-dart-sdk-upgrade.md`
