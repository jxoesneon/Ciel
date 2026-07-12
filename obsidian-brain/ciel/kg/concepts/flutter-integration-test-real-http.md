---
title: Flutter Integration Tests with Real HTTP and Widget Bindings
type: concept
tags: [concept, flutter, testing, x-seed]
created: 2026-07-13
status: active
---

# Flutter Integration Tests with Real HTTP and Widget Bindings

## Definition

A pattern for Flutter widget or integration tests that need both a real `HttpClient` and the Flutter widget binding. The default `TestWidgetsFlutterBinding` installs a fake `HttpOverrides` that makes every HTTP request return `400`, so tests that talk to a real loopback server must explicitly disable that override after initializing the binding.

## The Pattern

```dart
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  HttpOverrides.global = null;

  group('...', () {
    // widget or integration tests that hit a real local server
  });
}
```

## Why It Matters

- `ServicesBinding.instance` is required by code that loads assets or uses platform channels.
- `AutomatedTestWidgetsFlutterBinding` also overrides `HttpOverrides.global` with a fake client.
- Resetting `HttpOverrides.global = null` restores the real `HttpClient` so loopback servers and actual network calls work.

## When to Use

Use this when a test file spins up a real local HTTP server (e.g., an X-Seed addon server integration test) while still needing Flutter asset or service bindings.

## When Not to Use

Do not use this for pure unit tests or tests that should use a mock HTTP client. For those, prefer `mockito` / `MockClient` / `Client` injection.

## Related

- [[ciel/diary/2026-07-13-xseed-addon-server-binding-fix]]
- [[ciel/projects/X-Seed/X-Seed]]
- [[verification-commands]]
