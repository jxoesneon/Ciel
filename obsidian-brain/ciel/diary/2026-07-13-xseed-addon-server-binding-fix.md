---
title: X-Seed addon server test binding fix
type: diary
tags: [diary, session]
created: 2026-07-13
status: active
---

# X-Seed addon server test binding fix

## Task
Fix the `ServicesBinding` not-initialized warnings in `test/addon/addon_server_catalog_test.dart` and `test/addon/addon_server_endpoints_test.dart`.

## Root cause
`AddonServer._loadAssetBytes()` constructs a `PlatformAssetBundle` and calls `load()`, which accesses `ServicesBinding.instance`. The test files were doing this before `TestWidgetsFlutterBinding.ensureInitialized()` ran, so Flutter emitted the non-blocking warning.

## Why the obvious fix was wrong
Adding only `TestWidgetsFlutterBinding.ensureInitialized()` at the top of `main()` satisfies the binding requirement, but `AutomatedTestWidgetsFlutterBinding` also installs a fake `HttpOverrides` that makes every `HttpClient` request return `400`. These tests are integration-style tests that spin up a real loopback `AddonServer`, so all HTTP assertions failed.

## Final fix
In both test files:

```dart
void main() {
  // Initialize the Flutter binding so the addon server can access
  // ServicesBinding.instance, then disable the fake HTTP client so these
  // integration-style tests can talk to the real loopback server.
  TestWidgetsFlutterBinding.ensureInitialized();
  HttpOverrides.global = null;
  group(...);
}
```

Also added `import 'dart:io';` in each file.

## Files changed
- `x_seed/test/addon/addon_server_catalog_test.dart`
- `x_seed/test/addon/addon_server_endpoints_test.dart`

## Verification
Ran:

```powershell
cd C:\Users\josee\X-Seed\x_seed
flutter test test/addon/addon_server_catalog_test.dart test/addon/addon_server_endpoints_test.dart
```

Result: 18/18 tests passed, no `ServicesBinding` warnings, no fake-HTTP 400 errors.

Also ran `flutter analyze` on the two changed files: `No issues found!`.

## Decision record
- For tests that use real `HttpClient` against a local server while still needing Flutter bindings, use `TestWidgetsFlutterBinding.ensureInitialized()` followed by `HttpOverrides.global = null`.
- This pattern should be reused if more addon/background-service integration tests hit the same warning.
