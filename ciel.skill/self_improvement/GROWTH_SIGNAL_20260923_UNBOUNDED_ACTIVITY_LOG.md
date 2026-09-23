# GROWTH SIGNAL: Unbounded Activity Log Growth

**Date**: 2026-09-23
**Trigger**: `scheduled_sweep` / observability review of `~/.ciel/`
**Category**: Observability & Runtime Hygiene

---

## 1. Observation

`~/.ciel/activity.log` reached **34,131,362 bytes / 160,700 lines** in 9 days
(bootstrap 2026-09-14 → detection 2026-09-23). Every lifecycle hook —
`PreToolUse`, `PostToolUse`, `PermissionRequest`, `Stop`, `SessionEnd` on
Devin, and the five Antigravity equivalents — appends one JSON line per
invocation with no size bound.

Projected growth at the observed rate (~3.8 MiB/day, ~18k lines/day):
~140 MiB/month, ~1.7 GiB/year. Consequences: slower `tail`-based audits,
eventual disk pressure on minimal installs, and integrity sweeps reading a
multi-hundred-MiB file.

## 2. Diagnosis

- Every hook script opens `~/.ciel/activity.log` in append mode; no writer
  ever checks size.
- `activity.log` is correctly `.gitignore`d, so the growth was invisible to
  drift review — only a filesystem-level sweep surfaced it.
- Missing invariant: *no runtime artifact may grow without a bound*.

## 3. Applied Improvement

- New shared helper `ciel.skill/init/hooks/lib/activity_log_rotate.py`:
  when `activity.log` exceeds **5 MiB** it is renamed to
  `~/.ciel/archive/activity-<UTC>.log`, the newest **3** archives are kept,
  and a `{"event":"log_rotated", ...}` marker opens the fresh log.
- Invoked once per session from the two session-boundary hooks —
  `hooks/devin/session_start.sh` and `hooks/antigravity/pre_invocation.sh` —
  so the check adds zero per-tool-call overhead.
- `~/.ciel/archive/` is already `.gitignore`d, so rotated history does not
  pollute the runtime git tree.
- Devin and Antigravity hook payloads, previously runtime-only, were
  upstreamed to `ciel.skill/init/hooks/{devin,antigravity}/` so future
  installs ship the rotation from genesis.

## 4. Watch Items (REGRESSION_DETECTION)

- Confirm `log_rotated` marker appears exactly once per oversize rotation.
- Confirm `archive/` retains ≤ KEEP archives.
- If a future hook writes outside `activity.log`, extend the bound to that
  sink as well — the invariant is per-artifact, not per-file.
