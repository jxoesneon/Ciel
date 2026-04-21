# CHANGELOG

<!-- markdownlint-disable MD024 -->

All notable changes to Ciel are tracked here. Ciel appends an entry on every self-mutation commit. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) with SemVer.

## [Unreleased]

### Added

- Core skill integration system with individual Council review per `council/invocation_scopes/SKILL_INTEGRATION.md`.
- **MAJOR: Complete skill ecosystem integration and licensing**:
  - Integrated 230 skills from ECC ecosystem (plus 32 existing = 262 total)
  - Added Apache-2.0 LICENSE to all 262 skills
  - Created PROVENANCE.md for all 262 skills with audit trail
  - Established curated adaptation pipeline for content review
  - Fully adapted 2 skills with original content: skill-discovery, convoy-orchestration
  - 260 skills marked "PENDING ADAPTATION REVIEW" for content originality verification
  - Created adaptation workflow: Triage → Analysis → Harmonization → Original Creation → Council Review → Integration
  - Skills organized into priority tiers (Tier 1: Essential, Tier 2: Infrastructure, Tier 3-5: Domain-specific)
  - Risk assessment completed: 8 "direct-port" skills flagged for priority rewrite
- Trigger activation system with comprehensive pattern matching (direct/functional/domain/intent/composite).
- Local skill discovery from `~/.agents/skills/` with format detection (ciel-native, ecc-simple, generic).
- Harmonization pipeline: ECC Simple format → Ciel skill/1.0 format with backup/archival.
- `TRIGGER_REGISTRY.md` — Central trigger storage with confidence scoring.
- `TRIGGER_GENERATOR.md` — 6-stage pipeline for dynamic trigger generation.
- `LOCAL_DISCOVERY.md` — Foreign skill discovery and ingestion workflow.
- `ADAPTATION_MAPPINGS.md` — Runtime-to-Ciel field conversion reference.
- Scripts: `analyze-skill.sh`, `ingest-core-skill.sh`, `discover-local-skills.sh`, `generate-triggers.sh`.

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
- 33 seed skills covering filesystem, shell, git, web, MCP, package managers, code ops, testing, docker, API, docs, audit, environment, skill building, archive, memory, council, summarization, research, project analysis, runtime adapter building, diff/patch, config parsing, log analysis, secrets, linting, CI/CD, DB, markdown.
- Risk model with LLM judge for mid/high, user escalation for critical.
- Self-improvement loop with git rollback.
- Activity log + OTEL observability.
- Templates for skill, subagent, council vote, activity entry, adapter, config, risk assessment, improvement proposal.

### Notes

- This changelog is auto-maintained. Do not hand-edit entries below the `[Unreleased]` heading unless explicitly coordinated with Ciel's self-improvement loop.
