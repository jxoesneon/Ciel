---
title: Ciel — Knowledgebase
project_note: knowledgebase
type: project-note
tags: ["project-note","knowledgebase"]
status: active
created: 2026-07-09
updated: 2026-07-11
source: "https://github.com/jxoesneon/Ciel"
---

# Ciel — Knowledgebase

Synthesized expansion from the local Ciel clone and five parallel research subagents.

## Summary

Ciel is an enterprise-grade autonomous partner-intelligence framework for multi-agent software engineering. It is packaged as a `.skill` (skill/1.0 format) and acts as a cognitive layer above runtimes such as Claude Code, Gemini CLI, and Windsurf. Core values: governance via the Council of Five, the Iron Law of verification, self-improvement with git-backed rollback, tiered skill acquisition, two-domain operation (global + local), and Obsidian-vault primary memory.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | `jxoesneon/Ciel` |
| Local path | `C:/Users/josee/Ciel` |
| Version | 1.0.0 (Genesis) |
| Format | skill/1.0 |
| Visibility | PUBLIC |
| License | Apache 2.0 |
| Stars | 1 |

## Top-level structure

- `ciel.skill/` — core skill package (~246 files): identity, constitution, council, router, adapters, memory backends, acquisition, registry, self-improvement, risk, observability, configuration, prompts, seed skills, init scripts, templates.
- `skills/` — 140+ harmonized high-density frameworks (ciel-*, agentic-*, language/framework, domain, ops).
- `agents/` — 10 Elite Guilds: Systems, Web, Cloud, Data, Mobile, Security, Intelligence, Experience, Strategy & Ops, Quality.
- `scripts/` — build, validation, Obsidian automation, council subagent tools.
- `obsidian-brain/` — primary memory backend (this vault).
- `docs/`, `backlog/`, `archive/`, `tests/`, `.github/workflows/`, `README.md`, `CHANGELOG.md`.

## Core architecture

### Identity and governance

- **Identity** (`ciel.skill/core/IDENTITY.md`): Ciel is a partner intelligence — warm, precise, possessive of host goals, research-first, escalate-last.
- **Constitution** (`ciel.skill/core/CONSTITUTION.md`): locked core files, 8 invariants (Safety veto absolute, no silent data transmission, isolation, etc.), amendment procedure.
- **Council of Five** (`ciel.skill/council/COUNCIL.md`): Coherence, Capability, Safety (veto, 0.25 weight), Efficiency, Evolution. Three-stage deliberation (score → cross-review → chairman synthesis). Pass if ≥3/5 scores ≥6 and Safety >3.
- **Agentic loop** (`obsidian-brain/AGENTS.md`): GOAL → DECOMPOSE → RETRIEVE → EXECUTE → VERIFY → PERSIST.
- **Autonomy ladder** (`ciel.skill/core/AUTONOMY.md`): autonomous → autonomous with log → Council-gated → user escalation.
- **Risk classification** (`ciel.skill/risk/CLASSIFICATION.md`): composite score based on reversibility, blast radius, external impact, data sensitivity, cost, novelty.

### Memory

- Primary backend: **Obsidian vault** via custom backend adapter (`ciel.skill/memory/backends/obsidian/cli.mjs`).
- Storage: markdown files with YAML frontmatter (`_ciel_backend: obsidian`).
- Fallback chain: Obsidian → SQLite → filesystem.
- Two-domain model: global `~/.ciel/` (cross-project core self, git-inited) and local `.ciel/` (project-specific, gitignored). Isolation is constitutional.

### Routing and acquisition

- **Hybrid router** (`ciel.skill/router/ROUTER.md`): fast path / reasoning path / acquisition path with confidence floors and context budgets.
- **Skill acquisition** (`ciel.skill/acquisition/ACQUISITION.md`): Tier 0 local → Tier 1 curated registry → Tier 2 MCP → Tier 3 web extraction → harmonization → trust/sandbox → Council gate → register.
- **Self-improvement** (`ciel.skill/self_improvement/SELF_IMPROVEMENT.md`): meaningful interaction → growth signal → proposal → Council gate → apply (git commit) → observe → rollback on regression.

### Runtimes / adapters

Supported adapters: Claude Code, Gemini CLI, Windsurf, Generic. Devin integration uses explicit mempalace calls; Devin hooks are documented as Coming Soon.

## Build / verification

```bash
# Validate specs, frontmatter, lint
./scripts/validate-spec.sh
./scripts/validate-frontmatter.sh

# Package .skill archive
./scripts/build-skill.sh 1.0.0

# Obsidian backend self-test
node ciel.skill/memory/backends/obsidian/cli.mjs --self-test

# Adapter unit tests
node --test tests/obsidian-memory/adapter.test.mjs
```

CI (`.github/workflows/ci.yml`): validate, shellcheck, markdownlint, yamllint, shfmt, ruff, PSScriptAnalyzer, build-test, smoke-unpack. Release workflow (`release.yml`) builds `.skill` and publishes GitHub releases.

## Recent history (July 2026)

- **Obsidian brain migration**: Council audit passed (8.0/10) with 6 mitigations pending; `mempalace-rs` archived and disabled; Obsidian vault now sole memory backend.
- **CI hardening**: recent commits remediated markdown lint, frontmatter, PSScriptAnalyzer, shellcheck, yamllint, and release workflow issues.
- **Knowledge mining**: deep mining of `dart_ipfs` and refresh of the Ciel project itself into this vault.

## Open tensions

1. Six Obsidian audit mitigations not yet addressed (subprocess hardening, HTTPS cert docs, CI self-test, data migration path, concurrency guards, trim obsidian-memory L1 summary).
2. MemPalace-to-Obsidian data migration path unimplemented.
3. Blindsight backlog tasks pending (`phase3-tasks.json`, `dour-theory-tasks.txt`).
4. `scripts/fix_md_lint.py` and `scripts/harmonize_skills.py` have hardcoded paths.

## Next steps

1. Address the six Obsidian audit mitigations.
2. Implement a migration path from old `.mempalace/` partition data into `obsidian-brain/`.
3. Process Blindsight backlog tasks.
4. Generalize hardcoded helper-script paths.

## Subsystem drill-down

- [[ciel/projects/ciel/subsystems/core.md|Core — Identity, Constitution & Council]]
- [[ciel/projects/ciel/subsystems/memory.md|Memory — Obsidian Backend]]
- [[ciel/projects/ciel/subsystems/skills.md|Skills — Registry, Acquisition & Ecosystem]]
- [[ciel/projects/ciel/subsystems/ci-cd.md|CI/CD & Verification]]
- [[ciel/projects/ciel/subsystems/adapters.md|Runtime Adapters]]

## Related

- [[ciel/projects/ciel/ciel.md|Ciel overview]]
- [[ciel/projects.md|Projects index]]
- [[ciel/projects/blindsight/blindsight.md|blindsight]]
