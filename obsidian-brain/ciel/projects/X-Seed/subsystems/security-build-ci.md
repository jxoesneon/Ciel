---
title: X-Seed — Security / Build / CI
project_note: subsystem
type: project-note
project: X-Seed
tags: [subsystem, x-seed, security, build, ci, keystore]
status: active
created: "2026-07-12T07:44:58.464Z"
---

# X-Seed — Security / Build / CI

Identity and data protection, product flavors, and the continuous integration/release pipeline.

## Summary

X-Seed implements a tiered security architecture with Ed25519 identity keys protected by Android Keystore (StrongBox/TEE/software fallback), biometric gates for 8 critical actions, and root/jailbreak detection. The build system supports two product flavors (`play` for Google Play, `full` for GitHub sideload) with automated CI/CD via GitHub Actions. Security focuses on key protection, loopback-only addon server binding, and DMCA compliance infrastructure.

## Key files

### Security
- `x_seed/lib/src/features/security/keystore_service.dart` — Ed25519 key generation, tiered AES-256-GCM storage, export/import with biometric gates
- `x_seed/lib/src/features/security/biometric_gate.dart` — Biometric authentication wrapper using `local_auth`
- `x_seed/lib/src/features/security/critical_action.dart` — Enum defining 8 biometric-protected actions
- `x_seed/lib/src/features/security/root_detection_service.dart` — Interface for trust detection
- `x_seed/lib/src/features/security/jailbreak_root_detection_service.dart` — Production implementation via `jailbreak_root_detection`
- `x_seed/android/app/src/main/kotlin/com/jxoesneon/x_seed/KeystorePlugin.kt` — Android Keystore AES key generation/encryption
- `x_seed/lib/src/features/core/blocklist_service.dart` — DMCA blocklist enforcement
- `x_seed/lib/src/app.dart` — Root detection warning dialog on startup
- `docs/specs/SECURITY_SPEC.md` — Security specification (v0.2.0)

### Build/CI
- `x_seed/pubspec.yaml` — Dependencies, version `1.0.0-rc.1`, SDK `^3.10.0`
- `x_seed/android/app/build.gradle` — Gradle config, flavors, signing, ProGuard
- `x_seed/android/app/src/main/AndroidManifest.xml` — Permissions, `allowBackup="false"`, deep links
- `.github/workflows/ci.yml` — CI pipeline: test, build-android, release-android
- `build-android.ps1` — Windows build automation with PATH configuration for sodium
- `Makefile` — Docker-based Linux build environment
- `docs/specs/RELEASE_SPEC.md` — Release specification (draft)

## Ed25519 identity and Android Keystore

### Tiered storage strategy

- **Tier 1**: Hardware-backed StrongBox/TEE (AES-256-GCM key generated inside secure hardware).
- **Tier 2**: Software-backed Android Keystore (TEE on API 28+, software on older).
- **Tier 3**: Flutter Secure Storage fallback (platform-backed AES).

### Storage keys

- `xseed_identity_private_key_encrypted` — AES-encrypted private key (Tier 1/2)
- `xseed_identity_private_key` — Raw Base64 private key (Tier 3 or dual-key resilience copy)
- `xseed_identity_public_key` — Base64 public key
- `xseed_keystore_tier` — Stored tier integer (1, 2, or 3)

### Dual-key decision

When Tier 1/2 succeeds, X-Seed stores two copies: one encrypted by the dedicated AES key, another by `flutter_secure_storage`'s own Keystore-backed key. This provides resilience if the dedicated key is invalidated (e.g., OS downgrade), but both keys are lost if the Android Keystore is fully cleared (e.g., app data wipe). True recovery requires a user-managed BIP39 seed phrase (Sprint 9).

### KeystorePlugin constants

- `KEY_ALIAS = "xseed_identity_aes_key"`
- `TRANSFORMATION = "AES/GCM/NoPadding"`
- `GCM_TAG_LENGTH = 128`, `GCM_IV_LENGTH = 12` bytes
- Key size: 256 bits
- `setRandomizedEncryptionRequired(true)`
- **Intentionally no `setUserAuthenticationRequired`** — the background P2P service needs unrestricted access; biometric enforcement is at the application level.

## Biometric gate

**Critical actions (8 total):**

1. `exportIdentityKey`
2. `importIdentityKey`
3. `changeAddonPort`
4. `changeBootstrapNodes`
5. `disableBackgroundService`
6. `resetIpfsRepo`
7. `changeProviderMirrors`
8. `clearProviderCache`

**Configuration:**

- `biometricOnly: false` — allows PIN/Password fallback
- `stickyAuth: true` — keeps session alive across app backgrounding
- `sensitiveTransaction: true` — flags transaction as sensitive to the OS

**Error mapping:**

- `NotAvailable`/`NotEnrolled`/`PasscodeNotSet` → `BiometricResult.unavailable`
- User cancellation → `BiometricResult.cancelled`
- Success → `BiometricResult.success`
- Other → `BiometricResult.error`

## Root detection and jailbreak handling

- `JailbreakRootDetectionService` wraps `jailbreak_root_detection` plugin.
- Android: RootBeer + DetectFrida.
- iOS: IOSSecuritySuite.
- Results: `trusted`, `rooted`, `emulator`, `externalStorage`, `unknown`.
- On app startup, if result is not `trusted`, a warning dialog is shown but the app continues. No hard block — threat model assumes rooted-device adversaries can bypass app-level protections.

## DMCA / blocklist compliance

- `BlocklistService` loads from `assets/dmca_blocklist.json` (currently empty, version 1, 2026-06-28).
- Supports blocking by `infoHash` and `url`.
- Signature field exists but is not yet implemented.
- `RELEASE_SPEC.md` requirements: DMCA Agent registration, contact info, repeat infringer policy, remote blacklist signing key, no hardcoded infringing-provider URLs, `.gitignore` community plugin directory.

## Addon server security

Middleware pipeline (in order):

1. Proxy header strip — removes `x-forwarded-for`, `x-real-ip`, `forwarded`, etc.
2. Loopback-only middleware — rejects non-127.0.0.1/::1 connections.
3. Host validation — regex `^(127\.0\.0\.1|localhost|\[::1\])(:\d+)?$`
4. Rate limit — token bucket: 100 req/min per IP, 20 req/min per endpoint.
5. CORS — added on 2xx responses.
6. Log requests — standard shelf logging.

Binding: `InternetAddress.loopbackIPv4` on port 7979 (fallback 7980–7988).

## Data at rest and network security

- Identity private key: AES-256-GCM (software) or TEE/StrongBox (hardware).
- IPFS repo, SQLite, Hive, logs, watchlist: OS-level FBE (Android 12+) only; no additional app-level encryption for non-key data.
- `android:allowBackup="false"` — Ed25519 identity key must NOT survive app uninstall.
- Loopback HTTP (127.0.0.1:7979) permitted; all outbound provider scraping and Cinemeta API use HTTPS.
- Production builds enforce `android:usesCleartextTraffic="false"`; network security config allows cleartext only to 127.0.0.1, localhost, ::1.
- Timeouts: provider scrapers 5s, Cinemeta API 10s, IPFS gateway 30s.

## Product flavors

- **`play`** (`com.jxoesneon.xseed.play`): Google Play Store, Firebase enabled, metadata browsing only.
- **`full`** (`com.jxoesneon.xseed`): Direct APK, zero backend, full P2P/scraping/IPFS.

Entry points:

- `lib/main.dart` → `full` flavor, `NoOpMonitoringService`, GitHub update check enabled.
- `lib/main_play.dart` → `play` flavor, `FirebaseMonitoringService`, GitHub update check disabled.
- `main_common.dart` → shared bootstrap with flavor-injected monitoring service.

## Firebase configuration (play only)

- Firebase Core, Crashlytics, Analytics (~2.5 MB total).
- `google-services.json` placed in `android/app/src/play/` (gitignored).
- CI decodes from `GOOGLE_SERVICES_JSON_BASE64` secret.
- Google Services plugin applied conditionally in `build.gradle` only for the play flavor.

## Release signing

- Debug builds: local Android SDK debug keystore.
- Release builds: `android/app/release-key.jks` with env vars:
  - `XSEED_KEYSTORE_PASSWORD`
  - `XSEED_KEY_ALIAS` (default: "xseed")
  - `XSEED_KEY_PASSWORD`
- CI decodes keystore from `XSEED_KEYSTORE_BASE64` secret.

## ProGuard configuration

Comprehensive rules for:

- Flutter embedding and plugin registrant
- MethodChannel/EventChannel callbacks
- Riverpod, go_router, Firebase plugins
- All Flutter plugins (`local_auth`, `flutter_secure_storage`, `workmanager`, etc.)
- Native crypto classes (`javax.crypto`, `java.security`, `android.security.keystore`)
- Kotlin metadata and coroutines
- Removes verbose logging in release; keeps line numbers for Crashlytics symbolication

## CI/CD pipeline

GitHub Actions workflow (`.github/workflows/ci.yml`):

- `test` — `dart format --output=none --set-exit-if-changed`, `flutter analyze`, `flutter test --coverage`, upload to Codecov.
- `build-android` — debug APKs for both flavors on every push/PR.
- `release-android` — signed AAB (play) / APK (full) on version tags and manual dispatch.

Release checklist (from `RELEASE_SPEC.md`):

- All P0 providers pass live integration tests
- `dart analyze --fatal-infos` clean
- Unit test coverage >= 80% (currently 89.05%)
- Integration tests on emulator (API 31, 34)
- Physical device test: Samsung + Xiaomi (background service survival)
- Battery profiler: < 1.5% / hr idle drain
- Security audit: loopback binding, keystore access, biometric gate
- APK size per ABI < 40 MB (full flavor < 80 MB)

## Known issues and build quirks

- **Kotlin Gradle Plugin warning**: Flutter emits warnings about `app_settings`, `device_info_plus`, `workmanager_android`, and `jailbreak_root_detection` applying KGP. No KGP-compatible upgrades available yet.
- **Widget-test isolation**: Full `XSeedApp` exercised in `test/widget_test.dart` with `FakeBackgroundService` and fake root detection; heavy platform plugins overridden in tests.
- **Windows build path**: `flutter build apk --debug --flavor full` requires `C:\Program Files\Git\bin` in PATH for the `sodium` native asset build; `build-android.ps1` handles this.
- **Docker volume mounts**: Makefile mounts sibling IPFS and dart_quic repos at absolute paths; running `flutter pub get` inside Docker overwrites `.dart_tool/package_config.json` with Linux paths, so re-run natively after Docker use.
- **No flavor-specific manifests**: play/full flavors share the main manifest; differentiation is via `build.gradle` and entry points.

## Verification commands

| Command | Expected result |
|---------|-----------------|
| `flutter analyze` | `No issues found!` |
| `flutter test` | 1735 tests passing |
| `flutter test --coverage` | >= 80% line coverage (89.05% last run) |
| `flutter build apk --debug --flavor play` | success |
| `flutter build apk --debug --flavor full` | success |

## Related

- [[ciel/projects/X-Seed/knowledgebase.md|X-Seed — Knowledgebase]]
- [[ciel/projects/X-Seed/subsystems/addon-stremio.md|Addon / Stremio]]
- [[ciel/projects/X-Seed/subsystems/providers-scraper.md|Providers / Scraper]]
- [[ciel/projects/X-Seed/subsystems/ui-ux-routing.md|UI / UX / Routing]]
- [[ciel/projects/X-Seed/subsystems/background-services.md|Background Services]]
- [[ciel/projects/X-Seed/subsystems/ipfs-node.md|IPFS / libp2p Node]]
