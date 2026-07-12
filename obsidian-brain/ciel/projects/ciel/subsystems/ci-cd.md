---
title: "Ciel — CI/CD & Verification"
project_note: subsystem
type: project-note
tags: ["project-note","subsystem"]
status: active
created: 2026-07-11
updated: 2026-07-11
---

# Ciel — CI/CD & Verification

## GitHub Actions

### `.github/workflows/ci.yml`

Triggers: push/PR to `main`, `workflow_dispatch`.

Jobs:
- `validate` — `validate-spec.sh`, `validate-frontmatter.sh`, script executability.
- `shellcheck` — lint shell scripts at warning level.
- `markdownlint` — markdown lint (continue-on-error, configured via `.markdownlint-cli2.jsonc`).
- `yamllint` — YAML validation in `.github/` and `ciel.skill/`.
- `shfmt` — shell formatting (2-space indent, case-sensitive).
- `ruff` — Python linting + unit tests via `python3 -m unittest discover tests`.
- `psscriptanalyzer` — PowerShell linting with `.PSScriptAnalyzerSettings.psd1`.
- `build-test` — dry-build `.skill` archive and verify structure.
- `smoke-unpack` — unpack artifact and run `verify.sh` in a simulated `CIEL_HOME`.

### `.github/workflows/release.yml`

Triggers: tag push `v*.*.*`, `workflow_dispatch`.

Jobs:
- `build` — validate, lint, build `.skill`, compute SHA-256, smoke-install in Docker.
- `release` — create GitHub release with CHANGELOG excerpt, upload artifacts.

## Validation scripts

| Script | Purpose | Command |
|--------|---------|---------|
| `validate-spec.sh/ps1` | 265+ expected files, frontmatter, secrets scan | `.\scripts\validate-spec.ps1` |
| `validate-frontmatter.sh/ps1` | YAML frontmatter parsing, license audit | `.\scripts\validate-frontmatter.ps1` |
| `build-skill.sh/ps1` | Build deterministic `.skill` ZIP | `.\scripts\build-skill.ps1 -Version 1.0.0` |
| `verify.sh` | Dependency check, Obsidian self-test | `bash ciel.skill/init/scripts/verify.sh` |

## Tests

- `tests/obsidian-memory/adapter.test.mjs` — Node native tests for ObsidianMemoryBackend (mock Local REST API).
- `ciel.skill/memory/backends/obsidian/package.json` — `npm test` runs `node --test tests/*.mjs`.
- `tests/test_md_fixer.py` — unit tests for `scripts/fix_md_lint.py`.

## Helper scripts

- `scripts/fix_md_lint.py` — auto-fixes common markdown lint rules (MD009, MD012, MD022, MD026, MD030, MD031, MD032, MD040, MD047).
- `scripts/lint-fix.py` — fixes MD040 and MD060.
- `scripts/harmonize_skills.py` — normalizes runtimes and enriches domain tags.
- Both `fix_md_lint.py` and `harmonize_skills.py` currently have hardcoded paths.

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
