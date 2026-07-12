---
title: "Ciel — Skills (Registry, Acquisition & Ecosystem)"
project_note: subsystem
type: project-note
tags: ["project-note","subsystem"]
status: active
created: 2026-07-11
updated: 2026-07-11
---

# Ciel — Skills (Registry, Acquisition & Ecosystem)

## Top-level skills

- `skills/` contains ~100+ user-installed harmonized skills (kebab-case directories).
- Categories: `ciel-*` (core), `agentic-*`, `agent-*`, language/framework, domain-specific, ops.
- Disabled: `skills/mempalace-rs.disabled` (superseded by Obsidian).

## Seed skills

- `ciel.skill/seed_skills/` provides 33 cold-start skills loaded before acquisition populates the registry.
- Load order: filesystem → shell → environment_detection → git → archive_manager → json_yaml_toml_parser → markdown_processor → diff_patch → package_manager → obsidian-memory → secrets_manager → web_fetch/web_search → mcp_manager → api_client → docker → sandbox → skill_builder/skill_installer → council_runner → code ops → test_runner/linter_formatter → dependency_audit → documentation → cicd_integration → database_client → log_analyzer → context_summarizer → project_analyzer → research → runtime_adapter_builder.

## Registry & indexing

- **Registry:** `ciel.skill/registry/REGISTRY.md` — source of truth: filesystem `~/.ciel/skills/`, `index.json`, MemPalace partition, git history.
- **Indexing:** `ciel.skill/registry/INDEXING.md` — trigger trie, tag inverted index, description embeddings, contract fingerprint.
- **Schema:** `ciel.skill/registry/SCHEMA.md` — full metadata schema (id, version, triggers, tags, io_contract, source tier, dependencies, performance stats).
- **Query interface:** via `obsidian-memory` skill — by_trigger, by_tag, by_description (semantic), conflicts.
- **Coherence sweep:** `ciel.skill/registry/COHERENCE_SWEEP.md` — periodic rebuild of indices from source-of-truth.

## Skill format

- Required frontmatter: `name`, `version`, `format: skill/1.0`, `description`, `runtimes`, `license`, `tags`, `triggers`, `source`, `dependencies`.
- Triggers: pattern + confidence pairs.
- Tags: taxonomy includes scope, runtime, risk, domain, language.
- Source: tier 0 (seed), 1 (curated), 2 (MCP), 3 (web).

## Acquisition & installation

- **skill_installer** — installs `.skill` bundles, validates schema, checksums, registers. Council-gated.
- **skill_builder** — scaffolds and assembles new `.skill` ZIP bundles.
- **ACQUISITION.md** — tiered pipeline: local → curated registry → MCP → web extraction/synthesis → harmonization → sandbox → Council gate → register.
- **TIER_1_REGISTRY.md** — curated registries with ranking weights (tag overlap 35%, trigger overlap 25%, origin trust 15%, runtime compat 15%, license compat 10%).

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
