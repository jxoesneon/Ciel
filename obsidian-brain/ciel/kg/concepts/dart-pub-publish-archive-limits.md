---
title: Dart pub.dev Publish Archive Size Management
type: concept
tags: [concept, dart, publishing, pub-dev, ci]
created: 2026-07-11
status: active
---

# Dart pub.dev Publish Archive Size Management

## Definition

A workflow for keeping a Dart/Flutter package under the pub.dev 100 MB archive limit by curating `.pubignore`, removing tracked-but-ignored files from the git index, and resolving dependency conflicts before publishing.

## Common Triggers

- `dart pub publish --dry-run` fails because the package is over 100 MB.
- Large directories dominate the archive: `test/`, `packages/`, `.ciel/`, `.mempalace/`, Docker/k8s files, web assets.
- A file is tracked despite being gitignored (e.g., `test/interop/swarm.key`).
- Direct and override dependencies conflict, causing consumer build failures.

## The Pattern

1. **Identify archive size** with `dart pub publish --dry-run`.
2. **Add a `.pubignore`** that excludes non-essential directories:
   - `test/`, `packages/`, `.ciel/`, `.mempalace/`, `.github/`, `k8s/`, `helm/`, `web/`, Docker files, `*.ps1`.
3. **Remove tracked-but-ignored files** from the git index while keeping the working tree file:
   - `git rm --cached test/interop/swarm.key`
4. **Resolve dependency conflicts**:
   - If a package is needed as an override for local development but must not be a direct dependency, keep it only under `dependency_overrides`.
5. **Bump version, update `CHANGELOG.md`, tag, and publish**.
6. **Create the GitHub release** so downstream Docker/workflows get a semver tag.
7. **Re-run `--dry-run`** until it passes with only acceptable hints.

## Why It Matters

A published archive over the limit or with conflicting dependencies breaks consumers and CI. Proactive `.pubignore` hygiene keeps the published surface minimal and correct.

## Related

- [[ciel/diary/2026-07-11-publishing-state-restored]]
- [[ciel/projects/IPFS/IPFS]]
- [[verification-commands]]
