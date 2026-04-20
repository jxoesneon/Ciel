# DOMAINS — Two-Domain Model

Ciel operates in two domains. Both are always present; they coordinate per `memory/MERGE_SEMANTICS.md`.

## Global Domain — `~/.ciel/`

See `GLOBAL.md`.

- Ciel's true self.
- Cross-project learnings.
- Git-versioned.
- Primary MemPalace partition `ciel-global`.

## Local Domain — `<project>/.ciel/`

See `LOCAL.md`.

- Project-specific context, rules, learnings.
- Git-ignored.
- MemPalace partition `ciel-project-<hash>`.

## Interaction

- `PROMOTION.md` — local → global lifecycle.
- `ISOLATION.md` — cross-project guarantees (Constitutional).
- `MULTI_RUNTIME.md` — when Claude Code and Gemini CLI both use Ciel on the same project.

## Observations

- Global is invariant across sessions; only self-improvement mutates it.
- Local is small, targeted, and recreatable from global + project code at any time.
- A user can fully delete `.ciel/` and Ciel will rebuild it at next init.
- A user can fully delete `~/.ciel/` and Ciel will bootstrap anew; prior learnings are lost unless a backup existed.

## Common Pitfalls (documented for maintenance)

- Reading across partitions without `lift()` — forbidden by Constitution.
- Committing `.ciel/` accidentally — gitignore handled by `init/GITIGNORE.md`; Ciel defensively detects and warns.
- Two projects with identical absolute paths (containerized mounts) — partition hash accepts collisions; user is prompted if detected.
