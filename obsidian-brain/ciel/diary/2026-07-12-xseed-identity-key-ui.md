---
title: 2026-07-12 — X-Seed Identity Key Management UI
type: diary
date: 2026-07-12
session_id: run-xseed-identity-key-ui
project: X-Seed
tags: [diary, session, x-seed, flutter, ui]
status: active
created: "2026-07-12T00:00:00Z"
---

# 2026-07-12 — X-Seed Identity Key Management UI

## Summary

Replaced the Sprint 9 `settingsKeyPlaceholder` stub in X-Seed Settings with a real Identity Key management section protected by the existing `BiometricGate`. The work wired the existing `KeystoreService` (which already supported X-SEED-KEY-v1 export/import and BIP39 seed phrases) into the UI via a new `keystoreServiceProvider` and a `FutureProvider`-based `identityKeyStatusProvider`.

## What changed

- **Providers**: added `keystoreServiceProvider` and converted `identityKeyStatusProvider` to a `FutureProvider` that reports key presence and hardware-backed (Tier 1) status.
- **UI**: extracted identity key widgets into `lib/src/features/ui/settings/identity_key_section.dart`:
  - `IdentityKeyStatusTile` — shows present/missing and hardware-backed status.
  - `ExportIdentityKeyTile` — biometric-gated export of the X-SEED-KEY-v1 payload with a copyable dialog.
  - `ImportIdentityKeyTile` — biometric-gated import from either the X-SEED-KEY-v1 payload or a BIP39 seed phrase.
  - `ShowSeedPhraseTile` — biometric-gated display of the 24-word BIP39 seed phrase backup.
- **Shared auth helper**: moved the critical-action biometric auth flow into `critical_action_authentication.dart` so it can be reused by the new section and the existing `SettingsScreen`.
- **Localization**: removed `settingsKeyPlaceholder`, added `settingsIdentityKeyTitle`, `settingsIdentityKeySeedPhrase`, dialog titles, copy/close labels, hints, success/error messages, and seed-phrase warning in both English and Spanish. Ran `flutter gen-l10n`.
- **Tests**: updated `localization_coverage_test.dart`, `localization_exhaustive_test.dart`, and `settings_groups_test.dart`; added `FakeKeystoreService` to `test/ui/test_helpers.dart`; created `test/ui/settings/identity_key_section_test.dart` with 9 widget tests covering status display, export, import, and seed-phrase flows.

## Verification

- `flutter analyze --fatal-infos` — 0 issues.
- `dart format --set-exit-if-changed` — clean after formatting.
- `flutter test` — 2178 tests passed (up from 2130, +48 new tests).
- `flutter test --coverage` — line coverage 81.61% (≥80% threshold).

## Notes and next steps

- The existing `KeystoreService` already contained full BIP39 seed-phrase logic; no service-level changes were required.
- The UI treats Tier 1 as hardware-backed and Tier 2/3 as software/fallback-backed, matching `SECURITY_SPEC.md`.
- Future polish could include a dedicated "Generate identity key" action in Settings when no key exists (currently key generation happens on first app launch), and Android Share intent for the exported key.
