---
type: diary
tags: [diary, ipfs, readme, documentation]
created: 2026-07-11
title: "README Roadmap & Comparison review"
status: active
---

# README Roadmap & Comparison review

Reviewed `C:/Users/josee/IPFS/README.md` lines 590-650 (Contributing, Roadmap, Comparison with go-ipfs).

## Findings

- `Contributing` section was standard but lacked a pre-work issue step and docs update reminder.
- `Roadmap` had several stale statuses:
  - MFS is completed (v1.11.5).
  - IPFS Pinning Service API is implemented (`lib/src/services/pinning/`).
  - CLI/daemon, Docker images, interop test suite, and plugin ecosystem phase 1 are all complete per `ROADMAP.md` and project notes.
  - QUIC transport foundation exists in `packages/dart_ipfs_quic` but is still stabilizing; remains In Progress.
  - Mobile optimization remains In Progress.
  - GraphSync exists but server-side MVP is still being hardened; added to In Progress.
  - Gossipsub protobuf wire-format compliance is still outstanding; added to In Progress.
  - Filecoin integration is not tracked in `ROADMAP.md` or the codebase; removed from Planned.
- `Comparison with go-ipfs` table:
  - Added PubSub qualifier (Gossipsub).
  - Added GraphSync qualifier (server-side MVP).
  - Added MFS and Remote Pinning Service rows (both Yes/Yes).
  - Added QUIC Transport row as Partial/Yes.

## Output

Revised Markdown sections written to:
`C:/Users/josee/AppData/Local/Temp/ciel-readme-roadmap-compare.md`

## Next steps

- Aggregate all section revisions and apply a single coordinated edit to `README.md`.
- Verify all version references are aligned with `pubspec.yaml` (currently `1.11.6`; `lib/src/version.dart` still says `1.11.5` and may need a bump).
- Run `dart analyze` and `dart test` after the aggregated edit.
