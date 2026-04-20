# CONTEXT_FILES — Claude Code

Claude Code loads `CLAUDE.md` files in a layered hierarchy:

1. `~/.claude/CLAUDE.md` — global (applies to every session)
2. `<project>/CLAUDE.md` — project-wide
3. `<project>/<subdir>/CLAUDE.md` — scoped
4. `CLAUDE.local.md` — personal, git-ignored (optional)

Ciel respects this order (later wins) and injects her own blocks via anchored sections.

## Ciel Anchor

Ciel's block is delimited by stable markers, so it can be safely updated or removed:

```markdown
<!-- CIEL-ANCHOR:start -->
(content managed by Ciel — do not hand-edit)
<!-- CIEL-ANCHOR:end -->
```

Updates happen only through `init/INIT.md` re-run or an explicit `/ciel` operator command.

## What Ciel Injects

**Global** (`~/.claude/CLAUDE.md`):

- one-paragraph Ciel identity summary,
- slash-command hints,
- the reminder that every non-trivial request should pass through `/ciel` or free-form that Ciel's router detects.

**Project** (`<project>/CLAUDE.md`):

- detected language / framework / conventions,
- project rules from `configuration/local/rules.config.md`,
- any local skills registered to this project,
- escalation threshold override note.

## Parsing Protocol

At init and re-init, Ciel:

1. Locates the target CLAUDE.md.
2. If missing → creates with just the anchor block.
3. If exists with anchor → replaces between markers.
4. If exists without anchor → appends the anchor block at the end after a separator.
5. Verifies with a checksum recorded in `~/.ciel/INTEGRITY.json`.

## Read Path

Ciel also *reads* existing `CLAUDE.md` at init to extract host-provided rules (e.g. language conventions, forbidden ops). These populate `configuration/local/rules.config.md` on first init; subsequent inits reconcile via integrity check.
