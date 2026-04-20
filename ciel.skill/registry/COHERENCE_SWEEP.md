# COHERENCE_SWEEP

Periodic holistic check of the registry and `~/.ciel/` tree.

## Schedule

Default weekly. Configurable via `configuration/global/improvement.config.md.sweep_interval`. Also triggered:

- post-major-migration (e.g. MemPalace-rs upgrade),
- after any `conflict_resolution` commit,
- manually via `/ciel-registry --sweep`.

## Scope

1. **Filesystem** — every entry in `~/.ciel/skills/` matches an `index.json` entry and vice versa.
2. **Checksums** — recompute and compare to stored `checksum` per skill.
3. **MemPalace** — registry partition keys match index.
4. **Tags** — all tags in-taxonomy; out-of-taxonomy tags collected for Coherence review.
5. **Triggers** — no unmapped trigger tokens; no duplicate triggers across non-overlapping skills.
6. **Deprecations** — deprecated skills have not been referenced in the past sweep-interval → remove.
7. **Versioning** — git log linear per skill; no hidden merges or force-pushes.

## Actions

All findings packaged into a report. Each finding classified:

- **auto-fix** — obvious (stale checksum after external tool, orphaned tag) — Ciel fixes directly, commit with `coherence_sweep: auto-fix <n>` prefix.
- **council** — any conflict, merge, or drift resolution goes through `council/invocation_scopes/SKILL_CONFLICT.md`.
- **user** — anything that looks like user-edited-registry gets escalated.

## Output

`~/.ciel/registry/sweeps/<ts>.md` — human-readable report of the sweep. Appended to `activity.log`.

## Performance

Full sweep on 10k skills target: < 30s. Incremental sweep (since last): < 2s.
