---
title: X-Seed — Per-install addon HTTPS certificate
type: diary
status: active
created: "2026-07-13T00:00:00Z"
tags: ["diary","session"]
---

# X-Seed — Per-install addon HTTPS certificate

## Done

- Replaced the bundled `assets/selfsigned_combined.pem` self-signed certificate with a per-install generated RSA certificate for the optional local addon HTTPS listener.
- Created `lib/src/features/addon/addon_cert_manager.dart`.
  - Uses `basic_utils` to generate a 2048-bit RSA keypair and self-signed X.509 cert.
  - Persists the private key in `FlutterSecureStorage`.
  - Writes the certificate to the app-private support directory and returns a `SecurityContext`.
  - Falls back to the bundled asset if generation fails.
- Updated `lib/src/features/addon/addon_server.dart`:
  - `_startHttps()` now uses `AddonCertManager.getSecurityContext()`.
  - Removed empty-password key loading from the primary path.
  - Added an injectable `certManager` parameter.
- Added `test/addon/addon_cert_manager_test.dart` and `test/addon/addon_server_test.dart` covering generation, loading, idempotency, clearing, and fallback behavior.

## Verification

- `flutter analyze` on changed files: `No issues found!`
- `flutter test test/addon`: 140/140 passed.

## Blockers / next steps

- Full-project `flutter analyze` reports 19 pre-existing issues in `lib/src/features/security/keystore_service.dart` and `test/security/keystore_service_test.dart` (duplicate `exportSeedPhrase`/`importSeedPhrase` declarations). These block the full test suite and need to be resolved separately.
- Consider removing `assets/selfsigned_combined.pem` entirely once the generated-cert path is proven stable in production.
