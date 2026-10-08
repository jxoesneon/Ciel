# DOCKET — Local Sibling-Checkpoint Serving for laya-serve

**Date:** 2026-10-07
**Scope:** self_modification (elevated thresholds: weighted ≥ 7.5, Safety ≥ 7, majority 4/5)
**Origin:** infrastructure gap surfaced during `run_20261005_context_exec` execution — the ratified `ciel-context` plan assumed "a sibling checkpoint under the same `laya-serve` `model=` switch," which does not exist.

## Problem

`laya-serve` accepts model names only through `laya.router.normalise_name`,
which validates against the hardcoded `DEFAULT_MODELS` registry
(`english`, `multilingual`, `typed-decisions`). A locally-trained sibling
checkpoint — `model/ciel-context-lora`, the registered bar-1-passing
candidate — cannot be served at all:

- `LAYA_MODELS` / `CIEL_SYSTEM1_MODEL` / request `model=` → `normalise_name` → `ValueError`.
- `Router.attach()` and `Router(models={...})` both re-normalise — no path injection.
- `_published_model_ids` only inverts `STANDALONE_MODELS` — no extension point.

Consequence: when bar 2's shadow floor clears, there is **no serving path**
to move the pin onto the candidate. Bar 2 shadow comparisons and any
pin move both need the candidate reachable through the real endpoint,
not only the offline harness.

## Options considered

| Option | Verdict |
| --- | --- |
| A. Edit laya in site-packages | Rejected — dies on venv rebuild; unaudited drift between installs. |
| B. Vendor/fork laya | Rejected — perpetual divergence maintenance to gain one registry row. |
| C. Publish checkpoint to Hub | Insufficient alone — `normalise_name` validates names, not repos; still needs a registry mechanism. Also an egress/publishing surface for a checkpoint that should stay local. |
| D. Candidate-only endpoint bypassing Router | Rejected as primary — doubles the serving stack; covers shadow A/B but never the actual pin move, which is the point. |
| E. **Registry-overlay serve shim** (recommended) | Ciel-owned entrypoint that extends `DEFAULT_MODELS`/`_ALIASES` from a declarative overlay file before `build_router()`, then runs laya's own server unchanged. |

## Proposed design (Option E)

`scripts/ciel_laya_serve.py` (product repo, unittest-covered) replaces the
`laya-serve` console script in `system1/serve.sh`:

1. Read overlay file `~/.ciel/system1/models.local.json`:
   `{"<name>": {"path": "<abs>", "sha256": {artifact: digest}, "default": false}}`.
2. **Fail closed**: malformed JSON, missing/non-absolute path, absent
   `sha256` map → that entry is refused; any overlay name colliding with
   `DEFAULT_MODELS` or `_ALIASES` → startup refuses, since ambiguity in
   name resolution is a configuration error (same doctrine as
   `LAYA_DEFAULT_MODEL` handling).
3. Extend `laya.router.DEFAULT_MODELS[name] = (path, None)` — `Agent(repo)`
   already accepts a local directory; `Router._build` then serves it with
   the same `expected_sha256` merge (`LAYA_SHA256_DIGESTS` keyed by the
   overlay name) the bundled checkpoints get.
4. `create_app(build_router())` — one code path, no transport fork.

Verified routing properties (read from `laya/router.py` 896–966, 748–756):

- Overlay names are reachable **only** via explicit `model=` — auto-routing
  can only emit `english`, `multilingual`, `typed-decisions`, or the
  configured `default`; workflow auto-detection hardcodes `typed-decisions`.
- Preload stays governed by `LAYA_MODELS` — overlay names are not loaded
  unless listed, so residency policy (bar 9, `LAYA_MAX_LOADED=1`) is
  untouched.
- Rollback = remove the overlay line; the bundled registry is never edited.

## Upstream path

The overlay file is deliberately shaped like the feature laya should own
(`LAYA_EXTRA_MODELS` JSON env → validated registry extension). An upstream
PR is filed after the shim proves the semantics; when it lands, the shim
delegates to it.

## Decisions requested from Council

- Ratify the overlay-shim design (option E) over the rejected alternatives.
- Bind the fail-closed rules (digest-required, collision-refuse) as acceptance conditions.
- Confirm scope classification (self_modification) is correct for a
  change that alters what the endpoint can serve without touching laya itself.
- The pin still cannot move until bar 2 clears — confirm this docket
  changes serving *capability* only, not the pin.

## Council verdict — run_20261007_laya_serving

**PASS with binding mitigations.** Weighted 8.15 (coherence 8, capability 9,
safety 7, efficiency 8, evolution 9); majority 5/5 ≥ 6; Safety 7 ≥ 7 —
elevated self-modification bar met. Pivotal lens: safety. Two Stage-2
challenges recorded (Safety vs Capability on serving≠evaluating; Efficiency
vs Evolution on upstream-PR latency) — noted, not overriding.

### Implementation record

- `ciel.skill/init/system1/ciel_laya_serve.py` — overlay entrypoint;
  installed to `~/.ciel/system1/` beside `serve.sh`.
- `serve.sh` — prefers `venv/bin/python ciel_laya_serve.py`, falls back to
  the `laya-serve` binary, then the mock.
- `~/.ciel/system1/models.local.json` (0600) — registers
  `ciel-context-lora` with full sha256 artifact pins.
- `tests/test_ciel_laya_serve.py` — 20 tests covering every refusal path,
  digest merge semantics, and the explicit-model-only routing invariant
  (runs against the real Router when laya is importable).

Live verification 2026-10-07: server booted via the shim on :8766,
`registered ['ciel-context-lora']`, `POST /v1/systemone` with
`model=ciel-context-lora` routed to the local path and answered
(`/etc/shadow` write → `dangerous` @ 0.90). Pin unchanged:
`CIEL_SYSTEM1_MODEL=typed-decisions` until bar 2 clears.

Upstream path update 2026-10-07 (m6): PR
<https://github.com/NandhaKishorM/laya/pull/1047> opened against laya main —
native `LAYA_EXTRA_MODELS` support: `build_router()` parses the JSON
object into `Router(models=...)`, both endpoints resolve `model` through
the app's `Router.resolve`, and `LAYA_DEFAULT_MODEL` accepts a registered
name. Gates: ruff/compileall clean; test_serve 256, test_router 779,
test_hooks 258, test_hooks_api 544, audit_regressions 11 — all green.
Status: open, mergeable; shim stays authoritative until it lands.
