---
title: "2026-07-08 Catalog all jxoesneon GitHub projects in Obsidian brain"
type: diary
tags: [diary, github, catalog]
created: "2026-07-08T00:00:00.000Z"
updated: "2026-07-08T00:00:00.000Z"
status: active
---

# 2026-07-08 Catalog all jxoesneon GitHub projects in Obsidian brain

Created an Obsidian project catalog for every public/private repository owned by `jxoesneon`.

## Actions

- Queried GitHub via `gh repo list jxoesneon --limit 100` and extracted metadata for 60 repositories.
- Generated project overview notes for each repo at `ciel/projects/<repo>/overview.md` using the Obsidian Local REST API.
- Each note includes metadata table, topics, inferred use/scope, and links to the projects index.
- Updated `ciel/projects.md` with a sorted manual list of all 60 projects in addition to the existing Dataview table.
- Parallel subagents were also launched to process batches; their work overlapped and confirmed the same set of notes.

## Statistics

- Total repositories: 60
- Public: majority
- Private: blindsight, X-Seed, community-plugins, FerroTex-Desktop, Faithful, NMSEditSwitch, anura-forge, Zegion
- Notable high-activity repos: `mempalace-rs` (34 stars, 6 forks), `IPFS` (10 stars, 5 forks), `SeedSphere` (7 stars, 4 forks)

## Next steps

- Enrich high-priority project notes (e.g., `IPFS`, `Ciel`, `quic_lib`, `mempalace-rs`) with README excerpts, architecture decisions, and current goals.
- Keep project notes refreshed as repositories evolve.
