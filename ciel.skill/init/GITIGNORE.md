# GITIGNORE — Project `.gitignore` Injection

Ciel adds `.ciel/` to the current project's `.gitignore` at local init.

## Algorithm

1. Locate `.gitignore` in project root.
2. If missing, create with header:

```text

# Project .gitignore

```

1. Check for exact `.ciel/` line.
    - Present → no-op.
    - Absent → append:

```text

# Ciel local domain

.ciel/
```

1. Write atomically (tmp + rename).

## Idempotency

Subsequent runs detect the entry and do nothing.

## Do Not Touch

Ciel does not reorganise, sort, or deduplicate the user's existing `.gitignore` content beyond adding her single entry. The anchor pattern is the three lines above.

## Corner Cases

- If the project uses a non-standard ignore file (e.g., `.hgignore`, `.svnignore`), Ciel detects and adds to the matching one.
- If the project is not a git repo at all, Ciel skips this step and logs a notice — `.ciel/` is still created.
- Symlinked `.gitignore` is respected; Ciel follows and writes through.
- `.gitignore_global` (`core.excludesFile`) is respected read-only; Ciel does not modify global user git config.

## Removal

`ciel uninit` removes `.ciel/` and offers to remove the three gitignore lines. User confirms; by default, the gitignore lines are retained (harmless, avoids surprise diffs).
