---
title: "README verification & update"
project_note: goal
type: project-note
tags: ["project-note","goal"]
status: completed
created: 2026-07-11
project: IPFS
---

# Goal: Verify and update README.md by section

Orchestrate one subagent per logical section of `C:/Users/josee/IPFS/README.md`. Each subagent must:

1. Read its assigned section in full.
2. Cross-check factual claims against the current codebase (`pubspec.yaml`, `lib/`, `doc/`, `test/`, package READMEs).
3. Update outdated information (versions, API names, metrics, feature status, links).
4. Keep the tone, formatting, and conventions of the original section.
5. Return a Markdown snippet containing only the revised section and a short changelog.

## Acceptance criteria

- [ ] Every major README section reviewed by a dedicated subagent.
- [ ] All version references consistent with `pubspec.yaml` (currently `1.11.6`).
- [ ] Code snippets compile against current public API.
- [ ] Feature claims match actual implementation status.
- [ ] Links point to existing files or valid URLs.
- [ ] Aggregated edits applied to `README.md` with clean Markdown.
- [ ] `dart analyze` passes (or pre-existing issues only).
- [ ] Session recorded in `ciel/diary/`.

## Context

- Repository root: `C:/Users/josee/IPFS`
- README path: `C:/Users/josee/IPFS/README.md`
- Current package version: `1.11.6` (`pubspec.yaml`)
- Monorepo: `packages/dart_ipfs_core`, `packages/dart_ipfs_quic`
- QUIC transport is stabilizing; `quic_lib` 1.13.0 is the canonical QUIC library for this project.
- Known test state: 3232 passing, 5 skipped, 6 failing (environment-dependent interop).
- Coverage: 86.61%.

## Related

- [[ciel/projects/IPFS/IPFS.md|IPFS overview]]
- [[ciel/projects/IPFS/knowledgebase.md|IPFS — Knowledgebase]]
- [[ciel/projects/IPFS/build-test-ci.md|IPFS — Build, Test & CI]]
