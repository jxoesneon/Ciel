---
type: diary
tags: ["diary","session"]
title: "README Examples & Testing audit"
status: active
created: "2026-07-09T00:00:00Z"
---

# README Examples & Testing audit

Reviewed `C:/Users/josee/IPFS/README.md` lines 540–588 against the current repository state and the `build-test-ci.md` project note.

## Findings

- `example/verify_bridge.dart` no longer exists; removed from the Examples list.
- Several existing examples were missing from the README:
  - `example/main.dart`
  - `example/rpc_example.dart`
  - `example/simple_test.dart`
  - `example/test_p2p_setup.dart`
  - `example/wasm_main.dart`
  - `example/web_p2p_chat.dart`
  - `example/plugins/logging_observer/main.dart`
  - `example/plugins/metrics_emitter/main.dart`
- Testing counts were stale: README claimed 2326 passing; current state per `build-test-ci.md` is 3232 passing, 5 skipped, 6 failing.
- Static-analysis wording updated: target is 0 errors; warnings/infos may be tolerated if pre-existing.

## Actions

- Produced revised Markdown for the Examples and Testing sections.
- Wrote the revised sections to `C:/Users/josee/AppData/Local/Temp/ciel-readme-examples-testing.md`.
- Did not edit `README.md` directly per instructions.

## Next steps

- Apply the revised sections to `README.md` in a follow-up edit if approved.
