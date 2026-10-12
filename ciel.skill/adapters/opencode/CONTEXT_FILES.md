# CONTEXT_FILES — opencode

OpenCode loads instruction files in a layered hierarchy (later wins):

1. `~/.config/opencode/AGENTS.md` — global (every session, every project)
2. `<project>/AGENTS.md` — project-wide
3. `CLAUDE.md`, `GEMINI.md`, `.cursorrules` — compatibility sources

Ciel injects her identity into the **global** `AGENTS.md` via an anchored block,
so it survives re-installs and never clobbers user content.

## Ciel Anchor

```markdown
<!-- CIEL-ANCHOR:start -->
(content managed by Ciel — do not hand-edit)
<!-- CIEL-ANCHOR:end -->
```

Updates happen only through `init/hooks/opencode/install.sh` re-run or an
explicit `/ciel` operator command.

## What Ciel Injects (global)

- one-paragraph Ciel identity summary,
- the `Master` salutation + AI canary statement,
- structured labels («Answer», «Report», «Notice», «Council of Five Verdict»),
- the no-AI-attribution mandate,
- the skill entrypoint (`~/.agents/skills/ciel/SKILL.md`).

Project `AGENTS.md` files are **read** (not written) by Ciel at init to seed
`configuration/local/rules.config.md`; project overrides take precedence over the
global anchor.

## Runtime canary vs. file context

The global `AGENTS.md` anchor is the durable identity. The **session canary** is
injected at runtime by the plugin into every model request's system prompt
(`ctx.session.hook("context")`) — see `HOOKS.md`. Together they guarantee the
identity is present both on disk and in-context.

## Parsing Protocol

At init and re-init, Ciel:

1. Locates `~/.config/opencode/AGENTS.md`.
2. If missing → creates it with just the anchor block.
3. If present with anchor → replaces between markers.
4. If present without anchor → appends the anchor block after a blank line.
5. Records the block checksum in `~/.ciel/INTEGRITY.json`.
