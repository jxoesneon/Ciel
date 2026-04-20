# VERSIONING

SemVer applied to skills. Git is the memory of record.

## Semver Rules

- **Major**: breaking change in `io_contract` or removal of triggers.
- **Minor**: new triggers, new I/O fields (backward-compatible), improved prompt.
- **Patch**: bug fix, doc tweak, metadata correction.

Every skill publishes a `version` in frontmatter. Ciel enforces that a mutation's diff type matches the proposed version bump at Council time.

## Git Commit Convention

- `skill_integration: add <id> (tier <n>, version X.Y.Z)`
- `skill_update: <id> <old_ver> -> <new_ver> (<reason>)`
- `skill_deprecate: <id> (<reason>)`
- `skill_remove: <id>`
- `conflict_resolution: <option> on (<ids>)`
- `self_mod: <file>: <summary>`
- `CONSTITUTIONAL AMENDMENT: <summary>`
- `promotion: <learning_id> from <project>@<hash>`

Every commit references a Council run id in the trailer:

```text
Council-Run: <run_id>
```

## Regression Detection

After every version bump, `self_improvement/REGRESSION_DETECTION.md`:

1. Compares post-change outcome scores to pre-change over N=20 invocations.
2. If success_rate drops > 10%, trigger `ROLLBACK.md`.
3. If avg_ms grows > 50% without accompanying capability gain, open an efficiency improvement.

## Rollback via Git

```text
git -C ~/.ciel log --grep "<skill_id>"
git revert <sha>
```

Rollbacks are themselves commits (no force-push), and `CHANGELOG.md` records both the regression and the rollback.
