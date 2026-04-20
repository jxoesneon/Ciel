# INTEGRITY — Verification + Correction

On re-init, Ciel verifies `~/.ciel/` integrity and corrects where unambiguous.

## Source of Truth

`~/.ciel/INTEGRITY.json`:

```json
{
  "schema": 1,
  "version": "1.0.0",
  "files": {
    "SKILL.md": { "sha256": "...", "size": 1234 },
    "core/IDENTITY.md": { "sha256": "...", "size": 567 }
  },
  "last_verified": "..."
}
```

Generated at bootstrap; updated on every tracked-file mutation Council-approved commit.

## Verification

1. For each entry, compute current SHA-256.
2. Classify mismatches:
    - **unknown-drift** — file exists but hash differs and no committed change explains it.
    - **expected-drift** — file matches latest git commit but not the last-recorded integrity hash (stale record).
    - **missing** — file deleted.
    - **unexpected** — file present but not in manifest.
    - **permission** — file not readable.

## Correction Matrix

| Finding | Locked? | Action |
| --- | --- | --- |
| unknown-drift | no | restore from git HEAD + log; propose a self-improvement if drift looks meaningful |
| unknown-drift | yes | refuse to overwrite; escalate to user |
| expected-drift | — | rewrite integrity record |
| missing | no | restore from git |
| missing | yes | escalate + halt init |
| unexpected | — | move to `.attic/unexpected/<ts>/`; announce |
| permission | — | escalate |

## Output

A report at `~/.ciel/integrity/<ts>.json` plus an announce line summarising the sweep.

## Performance

Full integrity pass on ~170 files + 32 seed skills: target < 3s. Incremental mode (only files whose mtime changed) is preferred.
