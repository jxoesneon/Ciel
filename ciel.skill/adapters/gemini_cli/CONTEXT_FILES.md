# CONTEXT_FILES — Gemini CLI

Gemini CLI uses `GEMINI.md` for project context plus extension playbooks for additional layered instructions.

## Hierarchy

1. `~/.gemini/GEMINI.md` — global
2. `<project>/GEMINI.md` — project
3. extension playbooks (per-extension contribution)
4. `.gemini/GEMINI.local.md` — local personal (gitignored)

Later layers override earlier on key-level overrides; otherwise additive.

## Ciel Anchor

```markdown
<!-- CIEL-ANCHOR:start -->
(managed by Ciel)
<!-- CIEL-ANCHOR:end -->
```

Ciel injects in project `GEMINI.md` and optionally global. Updates via init or `/ciel` command.

## What Ciel Injects

**Global**:

- Ciel identity summary,
- command hints (`@ciel`, `@ciel-council`),
- pointer to `~/.ciel/`.

**Project**:

- detected context,
- `configuration/local/rules.config.md` excerpts,
- per-project skills,
- active extensions Ciel is aware of.

## Extension Interaction

Ciel reads installed extensions and their playbooks at init, noting any overlap with her own registry. Overlap is flagged to the Coherence member for Council review (proposing consolidation or harmonization).

## Parsing Protocol

Same anchor-based upsert as Claude Code (`adapters/claude_code/CONTEXT_FILES.md`). Checksums tracked; re-init reconciles drift without losing user hand-edits outside the anchor region.
