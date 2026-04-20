# REGISTRY — Skill Registry

The internal map of everything Ciel currently knows how to do.

## Source of Truth

- **Filesystem**: `~/.ciel/skills/<id>/` — the actual skill bundles.
- **Index**: `~/.ciel/registry/index.json` — fast lookup metadata.
- **MemPalace**: partition `ciel/registry/` — semantic-search over descriptions.
- **Git**: `~/.ciel/` is git-inited; registry changes are commits.

## Metadata Shape

See `SCHEMA.md` for the full schema. Per-skill:

```json
{
  "id": "git/SKILL.md",
  "version": "1.2.0",
  "description": "...",
  "triggers": ["git", "commit", ...],
  "tags": ["vcs", "local", "shell"],
  "io_contract": { "input": {...}, "output": {...} },
  "confidence_default": 0.9,
  "source": { "tier": 1, "origin": "seed" },
  "install_path": "~/.ciel/skills/git/",
  "hits": 1240,
  "success_rate": 0.98,
  "avg_ms": 34,
  "created": "...",
  "last_updated": "...",
  "checksum": "sha256:..."
}
```

## Query Interface

Exposed by `seed_skills/mempalace_manager/SKILL.md`:

- `registry.by_trigger(token)` — exact and fuzzy.
- `registry.by_tag(tag)` — tag intersection search.
- `registry.by_description(natural)` — semantic search via MemPalace.
- `registry.describe(id)` — full metadata.
- `registry.stats(id)` — live stats.
- `registry.conflicts()` — overlap search.

## Write Interface

Only called by:

- `seed_skills/skill_installer/SKILL.md` after a Council pass,
- `self_improvement/` after Council pass for mutation/deprecation,
- `registry/CONFLICT_DETECTION.md` for de-duplication actions (Council-gated).

Direct hand-edit is forbidden; Ciel rejects out-of-band edits at next integrity check and corrects.

## Consistency Invariants

- Every index entry points at an existing skill file.
- Every skill file is referenced in the index.
- All checksums match on-disk content.
- No two skills share an identical `id`.

Violations trigger `registry/COHERENCE_SWEEP.md`.
