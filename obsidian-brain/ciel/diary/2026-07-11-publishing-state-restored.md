---
title: dart_ipfs publishing state restored and v1.11.7 shipped
type: diary
date: 2026-07-11
tags: [diary, dart_ipfs, publishing, ci, release]
status: active
created: "2026-07-11T00:00:00Z"
---

# dart_ipfs publishing state restored and v1.11.7 shipped

## Request

User asked to review the current publishing state and Git tags history, then said "continue".

## Findings from the review

- Latest git tag was `v1.11.5` (commit `8202742`, 2026-05-27); HEAD was 16 commits ahead.
- GitHub latest release was `v1.11.4`; no release existed for `v1.11.5`.
- pub.dev had `dart_ipfs 1.11.5`, `dart_ipfs_core 1.11.5`, and `dart_ipfs_quic 0.2.0` all published ~2 days ago.
- The visible GitHub Actions `Publish to pub.dev` run for `v1.11.5` had failed with exit code 65 due to a missing CHANGELOG entry and dependency override hints, yet the package was on pub.dev anyway (likely a later manual or re-triggered publish).
- `dart pub publish --dry-run` on HEAD failed because:
  - the package was 109.6 MB (over the 100 MB limit), dominated by `test/` (456 MB) and `packages/` (84 MB);
  - `test/interop/swarm.key` was tracked while gitignored.
- The root `pubspec.yaml` had `xml` and `dart_udx` both as direct dependencies and as `dependency_overrides`. The direct `xml: ^7.0.1` conflicted with `port_forwarder`'s `xml ^6.5.0`, which broke the Flutter dashboard build.

## Actions taken

1. **`.pubignore` cleanup**: excluded `test/`, `packages/`, `.mempalace/`, `.ciel/`, `.github/`, `k8s/`, `helm/`, `web/`, Docker files, and `*.ps1`. This dropped the published archive below the 100 MB limit.
2. **Untracked `test/interop/swarm.key`** (kept working tree file, removed from git index).
3. **Removed direct `xml` and `dart_udx` dependencies** from `pubspec.yaml`, keeping them only as `dependency_overrides` for local development/CI. Updated the override comment to explain why this is necessary and that the publish workflow accepts the resulting hints.
4. **Bumped version to 1.11.6**, updated CHANGELOG, pushed, tagged, and published. This succeeded on pub.dev but the Flutter dashboard build failed, confirming the `xml` conflict.
5. **Fixed the pubspec regression**, bumped to **1.11.7**, updated CHANGELOG, and published again. `v1.11.7` is now the latest on pub.dev.
6. **Created GitHub releases** for `v1.11.6` and `v1.11.7` so the Docker workflow would produce semver-tagged images.
7. **Fixed Docker SBOM failure**: set `upload-release-assets: false` on `anchore/sbom-action` because the job lacked `contents: write`. The `v1.11.7` Docker release run succeeded and pushed `ghcr.io/jxoesneon/dart-ipfs:1.11.7` (and `:1.11`, `:1`, `:latest`).

## Verification

- `dart pub publish --dry-run` passes with 0 warnings, 2 hints (dependency overrides).
- GitHub Actions on `master`: Test, Build, Docs, CodeQL, Docker, and Interop Nightly all green.
- Publish workflow: green for `v1.11.7`.
- Docker release workflow: green for `v1.11.7`.
- pub.dev: `dart_ipfs 1.11.7` is latest; `dart_ipfs_core 1.11.5` and `dart_ipfs_quic 0.2.0` remain current.

## Blockers / next steps

- `dart_ipfs 1.11.6` is still available on pub.dev and is broken for consumers because it declared `xml ^7.0.1` directly. Consider yanking `1.11.6` via the pub.dev web UI.
- `port_forwarder` still constrains `xml ^6.5.0`, preventing a clean upgrade to `xml ^7.0.1` for package consumers. The long-term fix is to get `port_forwarder` updated or replace it.
