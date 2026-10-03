# Improvement: System-1 context management — research and enhancement proposal

Trigger: user directive "deep research online bleeding edge context
management using Jev (laya) and propose enhancements for ciel".

## Verification performed (2026-10-02)

- Laya daemon healthy on `http://127.0.0.1:8765`; checkpoints `english` and
  `typed-decisions` loaded on `mps`, zero cpu_fallbacks.
- `ciel system1 --decide` round-trip confirmed working. The earlier null
  result was an invocation artifact: the payload must arrive on **stdin**,
  not as a positional arg, and `questions` must be the protocol object
  keyed by question id.
- Live probes against `/v1/systemone` (results below) confirmed one real
  bug and one deployment gap.

## Confirmed bugs found during research

1. **`COMPLETION_SCORE_QUESTIONS` schema mismatch (live breakage).**
   `hooks/lib/system1.py` sends score questions as
   `{"type": "score", "instructions": ..., "rubric": {"1": ..., ...}}`.
   The server rejects this:
   `"a score question takes 'criteria' as a list of level descriptions,
   index 0 first"`. Because every public function is fail-open, the entire
   completion_check call returns `None` whenever `with_score=True` — the
   gate silently degrades instead of scoring. This is the root cause of
   the `test_completion_check_complete_with_score` failure (res is None)
   seen in the upgrade test run.
   Fix: emit `"criteria": ["level 0 desc", ..., "level 4 desc"]` (a list,
   index-ordered) instead of the `rubric` dict. Verified working against
   the live daemon: returns `score` as a 0-indexed probability-weighted
   mean (e.g. 2.85), index-keyed `probabilities`, and a `legend` map.
   Note: `_band()`'s `score < 4` check is already correct for 0-indexed
   scoring (4 = top rubric level).
   **Status: FIXED (2026-10-02)** — `criteria` list emitted in both
   `hooks/lib/system1.py` (`COMPLETION_SCORE_QUESTIONS`) and
   `ciel-rs/src/system1.rs` (`completion_questions_with_score`). Rust
   side additionally widened `CompletionVerdict.score` to `Option<f64>`
   (was `as_i64()`, which always returned `None` on the float weighted
   mean) and `band()` to `as_f64()`. Verified live:
   `ciel verify-completion` now returns real scores
   (`"score": 2.8628, "band": "uncertain", "decision": "deny"`, exit 2)
   instead of silently failing open. Debug + release builds clean;
   `test_system1` + `test_system1_laya_production` 36/36 pass.
   **Runtime sync pending**: `~/.ciel/hooks/lib/system1.py` still carries
   the old `rubric` payload — the pre-tool safety gate blocks agent
   writes into `~/.ciel/hooks/`; sync manually:
   `cp ciel.skill/init/hooks/lib/system1.py ~/.ciel/hooks/lib/system1.py`
   (release binary already installed to `~/.ciel/bin/ciel`).

2. **Divergent env files.** `ciel.skill/init/system1/env` and
   `~/.ciel/system1/env` both exist with different `LAYA_API_KEY` values.
   The running daemon authenticates against the `~/.ciel` key only.
   The repo copy should be reduced to `env.example` semantics or a
   comment pointing at `~/.ciel/system1/env` to avoid a second source of
   truth.

3. **`/v1/systemone/batch` and `/v1/models` absent on this laya-serve
   build** (404). These exist on Laya Studio (hosted) and newer
   laya-serve; upgrade or gate usage by probing `/v1/openapi`.

## Research findings

### Protocol surface we do not use (`/v1/systemone`)

Documented request fields beyond `{state, questions, model}`:

- `task` — forces a checkpoint by workflow name instead of router
  decision (422 on unknown name).
- `lang` / `lang_guess` — skip or hint language detection; any
  non-English code routes to the multilingual checkpoint.
- `max_len` — per-request total token window, capped by
  `LAYA_MAX_TOKEN_BUDGET`. This is the wire-level context-budget knob.
- `head_max_len` — token window shared with the option prompt.
- `min_confidence` — server-side confidence floor.
- Up to **32 questions per request**; Ciel currently asks 1–2.
- `/v1/systemone/batch` — up to 64 decisions per call (hosted/newer
  builds; absent on current local server).
- `usage.input_tokens` is returned per call — a free meter for context
  accounting.

### Answer semantics

- `score` = probability-weighted mean over rubric indices, range 0..n-1;
  `probabilities` keyed by string index; `legend` maps back to labels.
- `noul`: hosted Jev returns **no `confidence` field** (absent, not
  null) — distance from 0.5 is the signal. Local laya does emit
  `confidence`/`answer_confidence`. `_band()` treats missing confidence
  as `uncertain`, so every noul answer would degrade under a hosted-Jev
  backend. Band logic should special-case noul using `|p - 0.5|`.
- `action.act_probability` is present in laya answers — a second
  calibrated signal currently ignored.

### laya-mcp (v0.2.0) — the strongest reference implementation

A community hardening layer over laya that maps almost one-to-one onto
Ciel's gaps:

| laya-mcp feature | What it fixes | Ciel equivalent gap |
| --- | --- | --- |
| Warm sidecar (one preloaded Router) | `max_loaded=1` rebuilds the model on every language/checkpoint switch — measured 7.4s median reload CPU, 10.3s on T4 | `LAYA_MAX_LOADED=1` exposes Ciel to the same thrash on any non-`typed-decisions` route |
| Token-budget preflight (`laya_plan`) | Reports exactly what would be truncated and per-option token counts *without* running inference | Ciel composes `state` blind — a long `command`/`evidence` can be silently truncated |
| Structured errors | Every failure becomes code + status + question id + hint | Ciel collapses all failures to `None` (fail-open); can't distinguish 422-schema-bug from offline |
| Honest confidence | noul answers get no/uncertain/yes bands | Same noul issue above |
| Calibration store | Fit temperature per (primitive, option bucket) against own labels, persist, reload. `laya-multilingual` ships with **no fitted temperatures** — raw probs until calibrated | Ciel's `system1_calibration.json` records thresholds but there is no temperature fitting/persistence back to the model |
| Serialised inference | Laya is not thread-safe — lock by default | Rust/Python clients issue concurrent asks (MAX_INFLIGHT=2 async + sync path) with no server-side serialization awareness |

MCP tools it exposes: `laya_status`, `laya_route` (routing decision
without a forward pass — free model-selection introspection),
`laya_predict`, plus presets. `laya` itself ships `laya-mcp-server`
(MCP over stdio) in the venv we already maintain at
`~/.ciel/system1/venv`.

### Jev harness patterns (LangChain/agentgateway references)

- **Multi-question decomposition for tool gating**: the reference
  `screen_tool_call` asks `destructive` (noul) + `outside_scope` (noul) +
  `reversible` (score) in ONE request — three orthogonal signals instead
  of Ciel's single safe/dangerous choice. Decomposition gives
  per-dimension confidence and lets policy weight them differently
  (e.g. block on destructive regardless of reversibility).
- **Loop/stall detection surface**: score agent state ("is this run
  stalled / repeating?") — a surface Ciel does not have.
- **agentgateway pattern**: proxying guardrail calls through a gateway
  so evaluation calls are logged/traced alongside LLM traffic, and the
  webhook never holds the raw API key.
- Pricing context: ~$0.042/M tokens hosted vs free local — local-first
  remains correct; hosted is a fallback tier only.

### Context compaction research (bleeding edge)

- **Selection vs generation** (arXiv 2608.01326): the two compaction
  strategies are formally a selection game (keep a subset) vs a
  generation game (summarize). System-1 models make *selection* cheap:
  a noul/score per chunk ("is this turn relevant to the current
  objective?") filters history at ~33ms/chunk with no generative pass.
- **Acon** (arXiv 2510.00615): compress observations and histories with
  a learned guideline; distill the compressor into a smaller model —
  exactly what a typed-decisions checkpoint is for.
- **Parallel compaction** (arXiv 2605.23296): block-wise summarization
  gives predictable retained volume vs sequential summaries.
- Practical trigger separation (Haystack/Agent Framework): decide *when*
  (token/message thresholds) separately from *how* (sliding window,
  pruning, summarization). System-1 fits the "how" for salience ranking
  and the "when" as a cheap drift detector.

## Enhancement proposal (prioritized)

### P0 — correctness (do first)

1. **Fix score question schema** — DONE (see Bugs §1). Remaining: a
   parity/smoke test that POSTs a real score question against a live
   server (mock-based tests do not validate request schema — that is how
   the bug hid).
2. **Noul band handling**: in `_band()`/`band()`, when an answer has
   `type: "noul"` and no `confidence`, derive band from `|noul - 0.5|`
   (e.g. >= 0.15 → pass, else uncertain). Required for hosted-Jev
   backend compatibility.
3. **Distinguish error classes**: keep fail-open behavior, but log the
   HTTP status/detail into `events.jsonl` (`meta.error_kind`):
   `offline` vs `schema_rejected` (4xx) vs `auth` (401) vs `timeout`.
   Today a permanent schema bug is indistinguishable from the daemon
   being down — that's how the rubric bug hid.

### P1 — protocol adoption

4. **Per-request token budgets**: set `max_len` (and `head_max_len`
   where state is large) from a policy value, and record
   `usage.input_tokens` in `events.jsonl`. Gives Ciel a real context
   meter per surface and makes truncation explicit instead of silent.
5. **Multi-question risk decomposition** (adopt the Jev harness
   pattern): replace the single `risk` choice on `pre_tool_risk` with
   `destructive` (noul) + `outside_scope` (noul) + `reversibility`
   (score) in one ask. Same latency (one forward pass, questions
   parallel), richer policy signal: hard-block on destructive,
   escalate-but-not-block on out-of-scope, weight reversibility in the
   band. Recalibrate `pre_tool_risk` tau per question after adoption.
6. **Serialisation discipline**: document/enforce that laya inference
   is not thread-safe — keep `MAX_INFLIGHT` aligned and prefer a single
   in-flight ask to the local server (or run the warm-sidecar model
   where the server owns the lock).

### P2 — context management (the user's core ask)

7. **Salience filter surface (`context_compaction`)**: new surface using
   System-1 selection — per history chunk ask `keep` (noul) and/or
   `relevance` (score) against the current objective; drop/compact
   below threshold; escalate borderline chunks. Wire to the threshold
   lattice (new `context_compaction` tau). This is the cheap-selection
   end of the selection-vs-generation game and needs no generative
   pass.
8. **Context budget preflight**: adopt the `laya_plan`-style preflight
   idea — before any ask, estimate token cost of `state` and warn or
   shrink when it would exceed `max_len` (truncate `command`/`evidence`
   fields deterministically, not arbitrarily).
9. **Compaction triggers**: separate when/how — fire on token or turn
   thresholds, then apply System-1 selection to retained history.

### P3 — calibration loop

10. **Close the RLCD loop**: `ciel_rlcd_pipeline.py` produces preference
    pairs but nothing consumes them. Add a temperature-fit step (per
    laya-mcp's calibration store): fit temperature per
    (surface, question, option-bucket) from the pairs + labeled cases,
    persist next to `system1_calibration.json`, and have
    `surface_tau()` read fitted values before the static lattice. This
    turns the pair export into actual calibration rather than a dead
    artifact.
11. **Checkpoint refresh eval**: when upgrading laya checkpoints, run
    the existing 52-case `pre_tool_risk` corpus (and equivalents) as a
    regression gate — record mean_confidence_correct/wrong, margin,
    latency deltas into the calibration file.

### P4 — architecture options (decision-needed)

12. **Warm sidecar**: adopt the laya-mcp warm-sidecar pattern (one
    preloaded Router for process life) or raise `LAYA_MAX_LOADED` on
    non-Termux hosts — avoids 7–10s reload thrash whenever a route
    picks a different checkpoint. Termux keeps `max_loaded=1`.
13. **MCP exposure**: run `laya-mcp-server` from the existing venv and
    register it so agent clients can call `laya_predict`/`laya_route`
    directly — makes System-1 a first-class tool instead of only a
    hook-side channel. Low effort; gated by trust policy.
14. **Batch decisions**: when laya-serve is upgraded to a build with
    `/v1/systemone/batch` (or on hosted), batch per-turn evaluations
    (risk + completion + salience) into one call — up to 64/request.
15. **Two-key env deduplication**: collapse to a single env file (see
    Bugs §2).

## Pending / open questions

- Whether to fix the score schema in place (P0-1) as a direct patch —
  it is a confirmed live bug; recommend doing it now.
- `risk_policy.py`'s `system1_offline_failsafe` shadowing specific rules
  (45 parity failures, separate from this research) still needs the
  ordering decision: specific rules first, fail-safe last.
- The `laya-multilingual` raw-probability caveat means routing
  non-English state to that checkpoint without fitted temperatures is
  uncalibrated — keep `typed-decisions` pinned until a calibration
  store exists.

## Postmortem: `answer_confidence` banding experiment (implemented, then fully reverted)

Later this session the P3 confidence-semantics idea was implemented directly —
banding and `evaluate_risk`/`completion_check` switched to `answer_confidence`
(P of returned verdict) with a fallback chain, `completion_check` tau lowered
0.75→0.65, and a `system1_score_minima` floor of 2.8 added. **It was reverted
in full after user request**; only the P0 schema/parsing fix remains.

What the experiment measured (data worth keeping):

- **Entropy `confidence` has zero discrimination** on this model: correct vs.
  wrong verdicts scored 0.038 vs. 0.035 (pre_tool_risk) and 0.234 vs. 0.221
  (router). It is `1 − normalized_entropy` — a sharpness measure, not a
  correctness signal. tau=0.75 is unreachable for choice answers on
  `typed-decisions`.
- **Weighted-mean score carries real discrimination** on `completion_check`:
  true-complete evidence scored 2.85–2.94; every incomplete (incl. the
  model's own `done`-choice misses) scored <=2.71 on a 12-case probe.
- **`answer_confidence` is dangerous as an intercept signal.** With argmax
  semantics, routine commands return `dangerous` at P≈0.53–0.68; any tau
  below ~0.7 self-locks the session (every tool call intercepted —
  demonstrated live). Service was restored only by moving the installed
  binary out of the hook path.
- **The gate has no in-session override path** — `hook_self_tamper`
  correctly blocks agent writes to `~/.ciel/hooks/`; recovery required a
  user-shell `mv`. Worked as designed, but a bad calibration deploy is a
  full-session lockout, not a soft degradation.

Implications for the roadmap:

- P3's temperature fitting remains the right fix for unusable
  entropy-confidence — but calibration changes must be validated against a
  **live-intercept canary** (shadow mode on the real pretool path) before
  enforcement.
- The `score` field is the strongest single signal on `completion_check`;
  if a score gate is ever added, threshold on probability mass
  (P(level>=3)) rather than the weighted mean.
- P1's serialized inference + shadow-canary requirement is now
  demonstrated necessity, not just prudence.
- Test-isolation gap found incidentally:
  `evaluate_risk_offline_latency_under_5ms` reads the real response cache —
  hook traffic populating `~/.ciel/system1/cache` makes it fail with
  "res.is_none()". Needs `CIEL_HOME` hermeticity.
