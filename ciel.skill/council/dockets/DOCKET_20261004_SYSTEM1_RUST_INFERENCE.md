# DOCKET 2026-10-04 — System-1 Rust-Native Inference Server (ort + ONNX)

- **Scope:** SELF_MODIFICATION (non-locked; proposes a new `ciel-system1` Rust
  binary crate under `ciel.skill/init/ciel-rs/` and, optionally, a fallback
  `laya[onnx]` Python serve shim)
- **Thresholds:** pass_weighted_score 7.5, safety_min 7, majority_required
  4/5 at pass_score 6
- **Repo:** `/home/eduardo/Ciel`
- **Host env:** laya 0.3.26 live on CPU (2C/4T Haswell); `ciel-system1.service`
  active; warm latency ~0.85 s/decision, ~0.62 s/state batched; prewarm
  shipped (restart `ExecStartPost` + detached session-start spawn)

## Background

The user's standing directive: Rust-first for all new code, except where
technologically required otherwise; async + multithreaded; auto-detected
CPU/GPU operation; prewarm at session start. The client side is now
Rust-complete (`--warmup`, `ask_batch`, session-start spawn). The remaining
Python is the inference server itself — `laya-serve`, which always
instantiates the PyTorch `Router` regardless of backend env vars and
serialises inference through a deliberate single-worker
`ThreadPoolExecutor` + `asyncio.Lock` (serve.py) around a documented
non-thread-safe `Router` — not the GIL (torch releases it during ops).

Deep research of laya 0.3.26 internals found the acceleration paths exist
but are SDK-only: `backend="compile"` (torch.compile), `backend="onnx"`
(`ONNXAgent`), `backend="tilelang"` (TileLang + CUDA graphs). `laya-serve`
exposes none of them. Three verified external facts change the calculus:

1. **The ONNX export is real and reproducible.** `scripts/export_onnx.py`
   ships in the upstream laya repo (verified: present at
   `NandhaKishorM/laya/scripts/`, ~7 KB — repo-only, not in the wheel, so
   Phase 1 fetches it commit/sha-pinned); a second implementation,
   `export/export_onnx.py` in `receptron/laya` (MIT), also exists as a
   licensed fallback. `--quantize` emits an INT8 weight-only graph —
   vendor-documented ~2x vs eager, but measured on VNNI-class silicon and
   with vendor-reported per-tensor decision agreement of only ~67–83%:
   **INT8 is export-compare material only**, never a serving default.
2. **A non-Python Jev runtime is proven.** `@receptron/laya` serves the same
   checkpoint through ONNX Runtime from Node.js and claims output parity to
   four decimal places; `receptron/laya-onnx` on Hugging Face carries a
   pre-exported fp32 bundle (graph + weights + tokenizer + temperatures) for
   the `english` checkpoint.
3. **`ort` (Rust ONNX Runtime bindings) has execution-provider support.**
   CUDA / TensorRT / DirectML / CoreML EPs behind cargo features with
   ordered registration — but `is_available()` only reports *compiled-in*
   support, not session usability; auto-detect must key on
   `register()`/session-creation failure with CPU fallback. ORT sessions
   are thread-safe (contrast: laya's Router is not). Note: `ort`'s default
   `download-binaries` feature fetches prebuilt libonnxruntime at build
   time — forbidden under the no-silent-network mandate (see constraints).

## What a Rust server must reimplement (the parity surface)

The ONNX graph ends at logits. Everything around it is currently Python —
the **full** enumerated contract, verified against `laya.serve` 0.3.26:

- `build_sequence` collation — state + question set → input ids + option
  markers (tokenizer via the `tokenizers` crate reads `tokenizer.json`
  natively)
- logits → probabilities, per-cardinality temperature scaling
  (`temperature`/`temperature_by_options` from `rl_agent_config.json`,
  clamped), optional `binning_map` calibration, confidence,
  `act_probability`, noul-confidence semantics
- question semantics for `choice`/`noul`/`score`, `head_max_len`/`max_len`
  window splitting, `min_confidence` abstention
- the full request surface: `BODY_CONTROLS` = model, max_len,
  head_max_len, task, lang, lang_guess, min_confidence; batch controls
  batch_size + sort_by_length
- response fields: `answers`, `routing.model`, `action`,
  `answer_confidence`, `usage` (incl. `input_tokens`), `legend`
- error semantics with ordering: 400 (non-serializable state), 413
  (>50k-char serialized), 422 (validation incl. refusal keys), 500,
  503 (admission semaphore)
- numeric limits: MAX_QUESTIONS=64, MAX_BATCH_STATES=64,
  MAX_BODY_BYTES=2MiB, MAX_CHOICE_OPTIONS=100, MAX_SCORE_LEVELS=32,
  MAX_TOTAL_OPTIONS=512, 131072-token batch chunking
- `/health` dual-mode: unauthenticated `{"status":"ok"}`; authed →
  `loaded`, `revisions`, `device`, `device_is_preference`,
  `checkpoint_devices`, `cpu_fallbacks`
- model routing + `LAYA_MAX_LOADED` residency + `LAYA_IDLE_UNLOAD_SECONDS`
  + `LAYA_AUTO_TASK` task-detection
- redaction, key auth, `CIEL_SYSTEM1_*` contract — the clients already
  speak this wire protocol, so a conformant server is a drop-in.

A calibration-state audit is a Phase-1 deliverable: confirm whether a
`binning_map` file is deployed (none in the current env — confidence then
equals temperature-scaled `answer_confidence`) and prove every parameter
the Router applies is present in or exported alongside the artifacts.

The parity oracle already exists: run identical state/question corpora
through `laya-serve` and the Rust server and diff answers/confidences.

## Proposal under review (requires Council verdict)

**Phase 1 — artifact + host measurement.** Fetch upstream
`scripts/export_onnx.py` commit/sha-pinned (receptron MIT copy as licensed
fallback); export `typed-decisions` and `english` fp32 inside the existing
venv (`laya[onnx]` extra — hash-pinned like the rest). INT8 exported for
comparison only. Deliverables: max-logit-diff per checkpoint, the
calibration-state audit, and **host-measured** warm latency + RSS for
fp32-ONNX vs laya-serve on the target box (Python `onnxruntime` or the
Alt-B shim suffices for measurement) — this measurement gates Phase 2.

**Phase 2 — standalone Rust server** (name `ciel-system1d` — avoids the
`ciel-system1.service` unit and `ciel system1` subcommand collisions; a
separate package/workspace-member — never a `[[bin]]` inside `ciel`, whose
dep closure must stay lean). Ordered EP registration via session-creation
probing + CPU fallback; ORT intra-op threads capped at physical cores;
residency mirroring `LAYA_MAX_LOADED`/`LAYA_IDLE_UNLOAD_SECONDS`; same
`env` schema; in-process prewarm. Async-stack choice (axum+tokio vs a
sync threaded server matching repo idiom) recorded in a short ADR.
`rust-version` floor set by the new crate (1.75 is insufficient).

**Phase-2 go/no-go (measurement gate):** proceed only if fp32-ONNX is
≥1.5× warm-latency faster than laya-serve on this host AND ≤1.5 GB RSS
is achievable under `LAYA_MAX_LOADED` residency. If the gate fails, the
decision re-opens between Alt B (Python ONNXAgent shim — which inherits
laya's post-processing and shares all Phase-1 artifacts) and status quo —
it does not silently select Alt B.

**Phase 3 — parity + shadow.** Versioned parity corpus (under
`tests/fixtures/` convention) covering the full enumerated contract;
gate = identical choices AND identical abstain/escalate decisions,
|Δconfidence| ≤ 1e-3 fp32, zero band-flips across every calibrated tau in
`risk/system1_calibration.json`. Then a live shadow window: **≥500 dual-run
asks AND ≥7 days**, divergences appended to `events.jsonl` in the existing
redacted format; zero unexplained divergences on flag-relevant decisions
before cutover. `/health` gains an additive `backend` attestation field
without breaking existing fields.

**Phase 4 — deploy + teardown.** `CIEL_SYSTEM1_SERVER` switch in env;
the serve.sh dispatch branch must error on a missing Rust binary — it may
**never** resolve to `mock_server.py` on the Rust path. Rollback = flag
unset + unit restart (laya-serve stays the rollback unit). Post-cutover,
one clean shadow window, then the torch venv removal is decided by a
separate explicit docket — no permanent silent dual-install.

## Alternatives the Council may weigh

- **B — Python `ONNXAgent` serve shim.** Same ONNX artifact behind a thin
  FastAPI replacing the Router. Captures the ~2x CPU win with far less new
  code, but keeps Python+GIL and adds no GPU/EP story — an interim step,
  not the Rust-first end state.
- **C — candle-native inference.** Pure-Rust from safetensors, no ORT dep;
  but requires reimplementing ModernBERT + the marker head forward in
  candle — the highest parity risk and most code, for no capability the
  ort path lacks.
- **D — hosted Jev offload.** `jev-agent.com` is reachable post-fix; zero
  infra for GPU-class latency on latency-critical calls. Rejected as the
  primary answer: egress cost, availability coupling, and it abandons the
  offline-first property. Remains valid as an opt-in fallback tier.
- **E — status quo.** 0.85 s/decision on this box is liveable for
  async/context surfaces; but it caps every latency-sensitive use
  (pre-tool `context_select`, interactive routing) and leaves the
  Rust-first mandate half-done.

## Runtime-safety constraints (binding on any implementation)

- Fail-open preserved: any server fault → client `None` → caller fallback;
  System-1 may tighten, never loosen, a rule verdict.
- Wire-protocol conformance is mandatory — the existing Python and Rust
  clients must not be able to tell which server answered.
- Confidence parity is a correctness gate, not a nicety: temperature and
  calibration-binning semantics must match laya-serve within the agreed
  bound, including zero band-flips across every calibrated tau.
- **INT8 is export-compare-only by default** — vendor-reported ~67–83%
  per-tensor decision agreement makes identical-choice parity implausible,
  and the vendor's own warning is "do not use it where calibrated
  probability or confidence matters." INT8 may serve only if it produces
  identical choices AND identical abstain/escalate decisions across the
  full corpus on host-measured runs, AND it is required by the RSS gate.
  fp32 remains the shipping default.
- **Zero silent network anywhere in build or run path:** `ort` with
  `download-binaries`/`fetch-models` forbidden; libonnxruntime resolved
  from a vendored sha256-pinned binary or a distro-signed system package
  via `load-dynamic` at a fixed recorded path; `cargo build --locked`;
  build and runtime must succeed with networking disabled. The server
  itself makes no outbound calls of any kind.
- **Digest-verify-or-refuse at load** (Rust-side, new code — the existing
  verification lives inside laya's Python loader): sha256 of `.onnx`
  graph, external-data file, `tokenizer.json`, and the calibration
  manifest, extending the `LAYA_SHA256_DIGESTS` schema. Self-exported
  artifacts only — third-party HF bundles (`receptron/laya-onnx`) are
  test vectors, never served.
- Server hardening: loopback bind unless explicitly configured; unchanged
  key auth; request-body cap, batch ≤64, bounded read/write timeouts,
  connection limits; no raw-state logging (events.jsonl redaction parity);
  `CIEL_SYSTEM1_DISABLED` and `CIEL_SYSTEM1_MODE` honored; unverifiable
  artifacts → refuse to serve, never degraded inference, never the mock.
- `/health` attestation: additive `backend` field identifying the engine;
  all existing fields preserved verbatim.
- New Rust deps hash-pinned via Cargo.lock; CI dep-closure check that
  `ort`/`tokio`/`axum`/`tokenizers` never enter the `ciel` hook binary.
- Initial build ships CPU-EP only — no GPU EP weight distributed until a
  GPU host exists (EP-detection code may exist).
- Sibling surfaces ship atomically with any implementation: `SYSTEM1.md`,
  `env.example`, `serve.sh` dispatch, `warmup.sh`, the systemd unit, and
  `init/launchd/com.ciel.system1.plist` — both supervised platforms.
- The parity corpus is versioned; latency claims in the final report are
  host-measured, not vendor figures (realistic expectation here:
  ~0.3–0.5 s/decision warm — still short of the 1.5 s
  CONTEXT_SELECT_BUDGET only at small k; honest budgeting required).
- No publish/tag without the release gate: ~complete coverage of new code
  + Council sign-off on the release diff.

## Rollback

The ONNX artifacts and the new binary are additive: `ciel-system1.service`
continues to run laya-serve until the cutover flag flips, and rollback is
`systemctl --user restart ciel-system1` with the flag unset. The Python
fallback engine in hooks/lib is untouched.

## Open questions — resolved by Council consensus (RUN_20261004_SYSTEM1_RUST_INFERENCE)

1. INT8 → **export-compare-only by default** (serving requires identical
   choices + abstain/escalate parity AND RSS necessity; vendor agreement
   of ~67–83% makes this a high bar).
2. Crate shape → **standalone `ciel-system1d` package** (never a `[[bin]]`
   inside `ciel`; dep-closure CI check; own `rust-version` floor).
3. ORT resolution → **constraint, not question**: zero silent fetches —
   vendored sha256-pinned binary or distro-signed `load-dynamic` at a
   recorded path.
4. Shadow window → **required**: ≥500 dual-run asks AND ≥7 days,
   divergences logged to `events.jsonl` (redacted format — also feeds the
   deferred `ciel-context` data-quality docket).
5. Hosted-Jev fallback tier → **out of scope**, separate docket if wanted.

## Verdict (RUN_20261004_SYSTEM1_RUST_INFERENCE)

Stage 1 → Stage 2: Coherence 8→7 · Capability 7→7 · Safety 7→6 ·
Efficiency 6→7 · Evolution 8→8. No Safety veto.

Weighted score = 0.20·7 + 0.20·7 + 0.25·6 + 0.15·7 + 0.20·8 = **6.95**.

Clears the general bar (≥6.5 weighted, 5/5 ≥6, no veto) but **misses the
elevated self-modification thresholds** the docket declares (weighted
≥7.5, safety ≥7) → **DEADLOCK → Chairman meta-judgment.**

Adjudicated disputes: (a) export-script provenance — BOTH
`scripts/export_onnx.py` upstream (verified via GitHub API, ~7 KB) and
receptron's `export/export_onnx.py` (MIT) exist; Capability conceded;
(b) "serialises on the GIL" — corrected to single-worker executor +
non-thread-safe Router (serve.py:852-856); (c) `is_available()` reports
compiled-in support only — `register()`/session-creation probing adopted;
(d) Coherence's "/batch 404" wrinkle is stale — `/v1/systemone/batch`
live-verified on the 0.3.26 install.

**Chairman meta-judgment — CONDITIONAL PASS, phase-scoped.** The Council
is unanimous that Phase 1 (pin-fetched export, fp32 + INT8 artifacts,
calibration audit, parity corpus, host-measured latency/RSS) proceeds —
every member endorsed it unconditionally, the work is additive and shared
with Alt B, and it carries zero cutover risk. Phase 2 remains
**authorization-pending**: it proceeds only on the host measurements the
amended docket now gates on (≥1.5× warm-latency vs laya-serve AND ≤1.5 GB
RSS), after which the measured artifact returns to Council as new
evidence. Phases 3–4 bind as amended.

The score shortfall was driven by resolvable docket defects — factual
overstatements and an under-scoped parity surface — all corrected in this
revision, plus the now-binding controls (zero-silent-fetch ORT,
digest-verify-or-refuse, never-fail-toward-mock, /health attestation,
full-contract corpus, shadow window, INT8 demotion, RSS gate, CPU-EP-only
build, async-ADR, atomic sibling surfaces, teardown clause).

## Implementation status (post-verdict)

Authorized now: Phase 1 only — pinned export fetch, fp32 + INT8 artifacts
for `typed-decisions` + `english`, calibration-state audit, parity-corpus
scaffolding, and host-measured latency/RSS. Everything else awaits the
Phase-2 gate evidence and a re-vote.

## Phase-1 measurement record (2026-10-04, host = 2C/4T Haswell, no GPU)

Executed per the phase-scoped verdict. Export tooling: upstream
`scripts/export_onnx.py` pinned at commit `b09832bd`, file sha256
`3dc6eac8971358ad5531c7124f2e631ddd4da6890a8df8a6237051e56b4594e2`;
extras installed hash-pinned (`onnx 1.23.1`, `onnxruntime 1.30.0`,
`onnxscript 0.7.2` + transitives; Master's revised supply-chain floor:
≥24 h + no reported issues). Artifacts in `~/.ciel/system1/onnx/` with
recorded sha256 digests (digests.txt).

| Metric | laya-serve (torch eager, THREADS=2) | fp32 ONNX | INT8 ONNX |
|---|---|---|---|
| Warm single decision | ~0.85 s | 0.83 s (default threads); **0.52 s** @ OMP_NUM_THREADS=2 | ~0.42 s |
| Batch, per state (6 states) | ~0.62 s | ~0.83 s | — |
| Process RSS | 1.63 GB (2 ckpts resident) | ~1.97 GB (incl. torch import; pure ORT ~1.8 GB est.) | 1.25 GB |
| Parity vs laya-serve | — | **6/6 identical choices, Δconfidence 0.0000** (choice/noul/score/multi-q) | **conf collapses to 0.0000 — fails parity outright** |
| Load time | — | 15.6 s | — |

Calibration audit: `rl_agent_config.json` carries `temperature` +
`temperature_by_options` (choice:11+ clamped 0.1005→0.5 at load — same
clamp both engines); no `binning_map` deployed, so confidence parity =
temperature-scaled `answer_confidence` only.

**Phase-2 gate verdict: FAILS.** fp32-ONNX meets ≥1.5× only on the
single-call path (1.6×) while *regressing* on the batch path that drives
the context surfaces (0.83 vs 0.62 s/state), and breaches the ≤1.5 GB RSS
gate (~1.9 GB). INT8 fits RSS and is ~2× but collapses confidence — dead
on arrival, exactly as the vendor warned. Per the Chairman's judgment the
decision re-opens between Alt B and status quo: Alt B would buy ~0.33 s
on single calls but regress batch, keep Python, and solve nothing
structural — **status quo stands on this host**. The exported artifacts,
parity evidence (the oracle works), and digest pins are banked for a
future host where the gate re-evaluates (GPU/vNNI/AVX-512, or a RAM
budget that admits fp32 residency).
