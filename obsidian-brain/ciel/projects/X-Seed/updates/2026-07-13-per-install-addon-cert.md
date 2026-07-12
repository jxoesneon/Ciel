---
title: X-Seed — Per-install addon HTTPS certificate
project: X-Seed
tags: [update, x-seed, security, addon, certificate]
type: project-note
project_note: update
status: active
created: "2026-07-13T00:00:00Z"
---

# Per-install addon HTTPS certificate

Replaced the bundled `assets/selfsigned_combined.pem` self-signed certificate with a per-install generated certificate for the optional local HTTPS addon listener.

## What changed

- Added `lib/src/features/addon/addon_cert_manager.dart`.
  - Generates a 2048-bit RSA key pair and self-signed X.509 certificate on first use using `basic_utils`.
  - Stores the private key in `FlutterSecureStorage` under `xseed_addon_cert_private_key`.
  - Stores the certificate in secure storage and writes it to a file in the app-private support directory.
  - Exposes `Future<SecurityContext> getSecurityContext()` for `AddonServer`.
  - Falls back to the bundled `assets/selfsigned_combined.pem` asset if generation fails.
- Updated `lib/src/features/addon/addon_server.dart`.
  - `_startHttps()` now loads its `SecurityContext` from `AddonCertManager` instead of loading the bundled asset directly.
  - Removed the empty-password `usePrivateKeyBytes(certBytes, password: '')` from the primary code path.
  - Added an injectable `certManager` constructor parameter.
- Added tests:
  - `test/addon/addon_cert_manager_test.dart` — generation, loading, idempotency, clearing, and fallback.
  - `test/addon/addon_server_test.dart` — HTTPS listener starts with generated cert, fallback asset, and degrades gracefully when cert loading fails.
- Added `basic_utils: ^5.7.0` to `pubspec.yaml`.

## Verification

- `flutter analyze lib/src/features/addon/addon_server.dart lib/src/features/addon/addon_cert_manager.dart test/addon/addon_cert_manager_test.dart test/addon/addon_server_test.dart --fatal-infos` → `No issues found!`
- `flutter test test/addon` → 140 tests passed.

## Notes

- The bundled `assets/selfsigned_combined.pem` remains only as an emergency fallback.
- On a real Android device the generated per-install cert is used; in unit tests without mocked `FlutterSecureStorage`/`path_provider` the fallback asset is used.
- A full-project `flutter analyze` currently reports 19 pre-existing issues in `lib/src/features/security/keystore_service.dart` and `test/security/keystore_service_test.dart` (duplicate `exportSeedPhrase`/`importSeedPhrase` declarations and test API mismatches). These are unrelated to the certificate change.
