# CHANGELOG

All notable changes to Ciel are tracked here. Ciel appends an entry on every self-mutation commit. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with SemVer.

## [Unreleased]

### Added

- (Ciel writes new capability acquisitions here via `self_improvement/GLOBAL_IMPROVEMENT.md`.)

### Changed

- (Config tunings, router weight adjustments, rubric revisions.)

### Deprecated

- (Skills flagged for removal — will be removed after one minor version.)

### Removed

- (Skills removed from registry.)

### Fixed

- (Regression repairs from `self_improvement/REGRESSION_DETECTION.md`.)

### Security

- (Safety-member veto actions, trust downgrades.)

---

## [1.0.0] — Genesis

### Added

- Full spec tree as defined in `SKILL.md` and `MANIFEST.md`.
- Two-domain model (`~/.ciel/`, `.ciel/`).
- Master Router (hybrid: fast / reasoning / acquisition).
- Council of Five (Coherence, Capability, Safety, Efficiency, Evolution).
- Tiered acquisition (curated registry → MCP → web).
- MemPalace-rs primary memory backend with SQLite + filesystem fallback.
- Runtime adapters: Claude Code (full), Gemini CLI (full), Generic (probe-based).
- 32 seed skills covering filesystem, shell, git, web, MCP, package managers, code ops, testing, docker, API, docs, audit, environment, skill building, archive, memory, council, summarization, research, project analysis, runtime adapter building, diff/patch, config parsing, log analysis, secrets, linting, CI/CD, DB, markdown.
- Risk model with LLM judge for mid/high, user escalation for critical.
- Self-improvement loop with git rollback.
- Activity log + OTEL observability.
- Templates for skill, subagent, council vote, activity entry, adapter, config, risk assessment, improvement proposal.

### Notes

- This changelog is auto-maintained. Do not hand-edit entries below the `[Unreleased]` heading unless explicitly coordinated with Ciel's self-improvement loop.
