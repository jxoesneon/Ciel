---
title: "2026-07-09: Archive mempalace-rs skills and switch to full Obsidian mode"
type: diary
date: 2026-07-09
session_id: run-archive-mempalace-1
project: ciel
tags: [diary, session, obsidian, migration, mempalace]
status: completed
created: "2026-07-09T00:00:00Z"
---

# 2026-07-09: Archive mempalace-rs skills and switch to full Obsidian mode

## Summary

The Ciel project is now in full Obsidian mode. The legacy \`mempalace-rs\` skill and the \`mempalace_manager\` seed skill have been backed up and disabled; the Obsidian backend is the sole primary memory backend.

## Actions

- Backed up \`skills/mempalace-rs/\` to \`archive/skills/mempalace-rs/\` and renamed the active copy to \`skills/mempalace-rs.disabled/\`.
- Backed up \`ciel.skill/seed_skills/mempalace_manager/\` to \`archive/ciel.skill/seed_skills/mempalace_manager/\` and renamed the active copy to \`ciel.skill/seed_skills/mempalace_manager.disabled/\`.
- Updated active dependencies in \`ciel.skill/seed_skills/context_summarizer/SKILL.md\`, \`council_runner/SKILL.md\`, and \`SEED_SKILLS.md\` to point to \`skills/obsidian-memory/SKILL.md\`.
- Updated \`ciel.skill/registry/REGISTRY.md\` and \`COHERENCE_SWEEP.md\` to reference the Obsidian backend.
- Updated memory docs to describe Obsidian as the primary backend: \`memory.config.md\`, \`FALLBACK.md\`, \`INSTALL.md\`, \`HEALTH_CHECK.md\`, \`MEMORY.md\`, \`MEMPALACE.md\`, \`PARTITION.md\`, \`backends/obsidian/README.md\`, \`backends/SQLITE.md\`, \`backends/CUSTOM.md\`, \`domains/ISOLATION.md\`, \`domains/MULTI_RUNTIME.md\`.
- Rewrote init scripts to install Obsidian backend dependencies instead of \`mempalace-rs\`: \`install.ps1\`, \`install.sh\`, \`setup.py\`, \`verify.sh\`.
- Updated \`ciel.skill/MANIFEST.md\`, \`BACKUP.md\`, \`BOOTSTRAP.md\`, and both \`CHANGELOG.md\` files.
- Removed disabled \`mempalace_manager\` from \`scripts/validate-spec.sh\` and \`scripts/validate-spec.ps1\`.
- Ran \`node ciel.skill/memory/backends/obsidian/cli.mjs --self-test\`: all checks passed.
- Ran \`node --test tests/obsidian-memory/adapter.test.mjs\`: 6/6 tests passed.

## Decisions

- The legacy memory skills are preserved as archives/disabled copies; they are not deleted.
- Generic skill references to \`mempalace\` as the abstract memory invocation remain, because they now resolve to the Obsidian backend implementation.

## Next Steps

1. Continue updating any remaining project-specific skills that hard-code \`mempalace-rs\` commands.
2. Re-run the full validate-spec suite after the skill tree changes.
3. Migrate any old \`.mempalace/\` partition data into \`obsidian-brain/\` if needed.
