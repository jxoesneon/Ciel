---
type: diary
date: 2026-07-11
title: 2026-07-11 — Physical device search tests
status: active
created: "2026-07-11T00:00:00Z"
tags: ["diary","session"]
---

# 2026-07-11 — Physical device search tests

## What was done

1. Reconnected to the OnePlus CPH2583 via wireless ADB after the TLS session timed out (new port `33765`).
2. Discovered the app was blocked on the onboarding Terms of Service screen; user accepted it.
3. Added `integration_test` dependency and missing `assets/top100`/`assets/top100.json` declarations to `pubspec.yaml`.
4. Added deep-link query handling to `SearchScreen` so `xseed://search?query=...` auto-submits searches.
5. Ran live searches on the phone for "The Matrix", "Inception", "Naruto", "asdfghjkl", and "xyzqwerty12345".
6. Captured screenshots of each result set.
7. Updated the `dart-wireless-debug` skill changelog and created a project update note.

## Results

- Real queries returned relevant, filtered results.
- Nonsense query `xyzqwerty12345` correctly produced the empty state.
- `asdfghjkl` returned actual movies titled `asdfghjkl`, so it is not a good negative test.
- ADB coordinate taps remained unreliable on this device's Flutter UI; deep links were the viable automation path.

## Files changed

- `lib/src/features/ui/search/search_screen.dart` — deep-link query handling
- `pubspec.yaml` — `integration_test` dev dependency, top100 assets
- `integration_test/search_phone_test.dart` — exploratory integration test (kept for future use)

## Next steps

- Consider keeping the deep-link search feature as a user-facing capability.
- Decide whether to keep or remove the exploratory integration test.
