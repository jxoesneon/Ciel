# GIT_SETUP — ~/.ciel/ Repository

Git-inits `~/.ciel/` on first run and maintains it thereafter.

## Initial Setup

```bash
cd ~/.ciel
git init
git checkout -b main
echo ".cache/" > .gitignore
echo "activity.log" >> .gitignore      # append-only — handled separately
echo "backups/" >> .gitignore
echo "archive/" >> .gitignore
echo "fs_backend/" >> .gitignore       # fallback backend data
echo "*.db" >> .gitignore              # sqlite fallback
echo "checkpoints/" >> .gitignore
echo ".attic/" >> .gitignore
echo "sandbox/" >> .gitignore

git add -A
git commit -m "genesis: Ciel cold start @ <version>"
```

## Remote

Not configured by default. Users may add one for backup; Ciel never pushes without explicit user action.

## Commit Message Convention

See `registry/VERSIONING.md`. All Ciel-initiated commits carry:

```text
<prefix>: <summary>

<optional body>

Council-Run: <run_id>                  # when applicable
Trigger: <self-improvement|user|sweep>
```

## Branches

- `main` — the live Ciel self.
- `self-mod/<timestamp>` — tags for rollback points after self-modification commits.
- Feature branches are not used; Ciel's changes are small and linearized.

## `activity.log` Handling

`activity.log` is gitignored (append-only, large). Its significant events are mirrored as summaries into committed files. For full history, users inspect the log file.

## Hooks

Git hooks under `~/.ciel/.git/hooks/` are **not** auto-populated. Ciel's self-discipline does not depend on git hooks (they are a user productivity tool). If Ciel needs a pre-commit check, she runs it inline rather than in a hook.

## Backup

See `BACKUP.md` — separate snapshot mechanism complementary to git.
