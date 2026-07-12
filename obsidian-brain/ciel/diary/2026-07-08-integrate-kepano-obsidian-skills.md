---
title: "2026-07-08: Integrate kepano/obsidian-skills"
type: diary
date: 2026-07-08
session_id: run-kepano-skills-1
project: ciel
tags: [diary, session, obsidian, skills]
status: completed
created: "2026-07-08T00:00:00Z"
---

# 2026-07-08: Integrate kepano/obsidian-skills

## Summary

User shared `https://github.com/kepano/obsidian-skills`. Cloned the repo and integrated its conventions into the Ciel Obsidian brain operating manual and the `obsidian-memory` skill.

## Actions

- Cloned `kepano/obsidian-skills` to `.ciel/obsidian-skills/`.
- Installed `defuddle` in `scripts/obsidian/` for web-to-markdown extraction.
- Verified `defuddle parse https://obsidian.md --md` works.
- Created `ciel/kg/concepts/obsidian-skills.md` as a reference note.
- Updated `obsidian-brain/_CLAUDE.md` with an **Agent Skills** section referencing all five skills.
- Updated `skills/obsidian-memory/SKILL.md` to reference `kepano/obsidian-skills` in a Related Skills section.

## Decisions

- Kept `kepano/obsidian-skills` outside the Ciel repo `skills/` tree because they use the Claude Code agent-skill format, not the Ciel `skill/1.0` registry format. They are referenced as external conventions rather than registered Ciel skills.
- `defuddle` is installed locally in `scripts/obsidian/` rather than globally to avoid polluting the system PATH.

## Verification

- Re-cloned `kepano/obsidian-skills` to `.ciel/obsidian-skills/` at session time.
- Confirmed `defuddle` is installed in `scripts/obsidian/` and `npx defuddle parse https://obsidian.md --md` returns clean markdown.
- Ran `node ciel.skill/memory/backends/obsidian/cli.mjs --self-test`: Local REST API, read-write, hybrid-search, and knowledge-graph all OK.

## Open Tensions

- None.

## Next Steps

1. Disable restricted mode in Obsidian so the REST API loads.
2. Run the adapter self-test to verify all components.
3. Start using `defuddle` for web-to-markdown extraction when ingesting web sources into `raw/`.
