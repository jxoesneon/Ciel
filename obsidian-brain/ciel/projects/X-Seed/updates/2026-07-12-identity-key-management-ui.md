---
title: Identity Key Management UI
date: 2026-07-12
project: X-Seed
type: project-note
project_note: update
tags: [project-note, update, x-seed, flutter, ui]
status: active
created: "2026-07-12T00:00:00Z"
---

# Identity Key Management UI

**Status**: completed  
**Verification**: `flutter analyze --fatal-infos` clean, `flutter test` 2178/2178 passed, coverage 81.61%.

## Changes

- `lib/src/features/providers/app_providers.dart`
  - Added `keystoreServiceProvider`.
  - Converted `identityKeyStatusProvider` from a static `Provider` to a `FutureProvider` that reads key presence and tier from `KeystoreService`.
- `lib/src/features/ui/settings/identity_key_section.dart` (new)
  - Public widgets: `IdentityKeyStatusTile`, `ExportIdentityKeyTile`, `ImportIdentityKeyTile`, `ShowSeedPhraseTile`.
  - Implements copyable export dialog, import dialog (auto-detects X-SEED-KEY-v1 vs BIP39 seed phrase), and seed-phrase backup dialog with an offline-storage warning.
- `lib/src/features/ui/settings/critical_action_authentication.dart` (new)
  - Extracted the shared biometric-gate helper used by critical settings actions.
- `lib/src/features/ui/settings/settings_screen.dart`
  - Replaced placeholder `_IdentityKeyStatusTile`, `_ExportKeyTile`, `_ImportKeyTile` with the new public section widgets and added `ShowSeedPhraseTile`.
- `lib/l10n/app_en.arb` and `lib/l10n/app_es.arb`
  - Removed `settingsKeyPlaceholder`.
  - Added `settingsIdentityKeyTitle`, `settingsIdentityKeySeedPhrase`, and dialog-specific strings.
- `test/ui/test_helpers.dart`
  - Added configurable `FakeKeystoreService` and a default override in `testApp`.
- `test/ui/settings/identity_key_section_test.dart` (new)
  - 9 widget tests for status display, export/import, biometric gating, and seed-phrase flow.
- `test/ui/settings_groups_test.dart`, `test/l10n/localization_coverage_test.dart`, `test/ui/localization_exhaustive_test.dart`
  - Updated expectations to remove `settingsKeyPlaceholder` and include new strings.

## Decision notes

- Kept the existing `KeystoreService.exportSeedPhrase()` / `importSeedPhrase()` signatures (no `biometricVerified` parameter); biometric enforcement is applied at the UI layer via `authenticateForCriticalAction`, consistent with how other non-service-critical UI actions are gated.
- Used package-relative imports (`package:x_seed/l10n/app_localizations.dart`) in the new UI files to avoid brittle `../../../` paths.
