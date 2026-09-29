# Improvement: close integration gaps from brag acquisition

Trigger: user directive "self improve" following skill_integration of brag/brag-slim.

## Changes
- scripts/build_registry_index.py — new; builds registry/index.json from
  filesystem truth (SKILL.md frontmatter + ciel.yaml), atomic write.
  Installed to ~/.ciel/scripts/. Result: 179 indexed.
- registry/index.json — created (was absent; documented invariant unmet).
- skills/council_runner/ — created; five member personas + chairman +
  rubrics sourced from ciel.skill/council/. Repairs the
  .devin/rules/ciel.md reference to ~/.ciel/skills/council_runner/.
- skills/hyperframes{,-core,-animation,-creative,-keyframes,-cli,-audio,-registry,-studio}
  — acquired (tier 3, heygen-com/hyperframes @ea48936, Apache-2.0);
  Council PASS 7.45 weighted. Closes brag's missing dep chain.
- skill_scan_baseline.json — 2 individually-reviewed findings recorded
  (hyperframes-creative REMOTE_CODE_EXEC, hyperframes-cli SECRET_ACCESS;
  both documentation-only).
- brag, brag-slim ciel.yaml — state: sandboxed (formal trust state;
  paired-eval pending).

## Council
Two verdicts logged to ~/.ciel/logs/council.log (SKILL_INTEGRATION PASS,
SELF_MODIFICATION PASS).

## Anomalies (not fixed — need decision)
- skills/design-system/ nests the real skill one level deep
  (design-system/design-system-skill/SKILL.md) — unindexed.
- skills/godogen/ and skills/mempalace/ have no SKILL.md — partial
  installs, unindexed.

## Pending
- paired_eval for brag/brag-slim/hyperframes family — requires
  evals/tasks/ set + nested `devin -p` runner sessions; gate remains
  open (state stays `sandboxed` until evidence lands).
