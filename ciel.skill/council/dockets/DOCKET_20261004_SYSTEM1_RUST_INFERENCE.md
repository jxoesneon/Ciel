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
serialises inference on the GIL.

Deep research of laya 0.3.26 internals found the acceleration paths exist
but are SDK-only: `backend="compile"` (torch.compile), `backend="onnx"`
(`ONNXAgent`), `backend="tilelang"` (TileLang + CUDA graphs). `laya-serve`
exposes none of them. Three verified external facts change the calculus:

1. **The ONNX export is real and reproducible.** `scripts/export_onnx.py`
   ships in the upstream laya repo; `--quantize` additionally emits an INT8
   weight-only graph documented at **~2x faster than eager on CPU** and
   ~1.8x faster than the fp32 graph. Max logit difference vs the PyTorch
   reference ≈ 1e-5.
2. **A non-Python Jev runtime is proven.** `@receptron/laya` serves the same
   checkpoint through ONNX Runtime from Node.js and claims output parity to
   four decimal places; `receptron/laya-onnx` on Hugging Face carries a
   pre-exported fp32 bundle (graph + weights + tokenizer + temperatures) for
   the `english` checkpoint.
3. **`ort` (Rust ONNX Runtime bindings) has first-class execution-provider
   detection.** CUDA / TensorRT / DirectML / CoreML EPs behind cargo
   features, `is_available()` probing, ordered EP fallback chains — exactly
   the auto-detect contract requested, and genuinely multithreaded (ORT
   intra-op pools + no GIL).

## What a Rust server must reimplement (the parity surface)

The ONNX graph ends at logits. Everything around it is currently Python:

- `build_sequence` collation — state + question set → input ids + option
  markers (tokenizer via the `tokenizers` crate reads `tokenizer.json`
  natively)
- logits → probabilities, per-cardinality temperature scaling, histogram
  binning calibration, confidence, `act_probability`
- question semantics for `choice`/`noul`/`score`, `head_max_len` window
  splitting, `min_confidence` abstention
- the Jev wire surface: `/health`, `/v1/systemone`,
  `/v1/systemone/batch` (≤64), model routing + `LAYA_MAX_LOADED` residency,
  `LAYA_AUTO_TASK`
- redaction, key auth, `CIEL_SYSTEM1_*` contract — the clients already
  speak this wire protocol, so a conformant server is a drop-in.

The parity oracle already exists: run identical state/question corpora
through `laya-serve` and the Rust server and diff answers/confidences.

## Proposal under review (requires Council verdict)

**Phase 1 — artifact.** Export `typed-decisions` (and `english`) to ONNX via
upstream `export_onnx.py` inside the existing venv (`laya[onnx]` extra:
`onnx`, `onnxruntime`, `onnxscript` — hash-pinned like the rest). Record
max-logit-diff evidence per checkpoint. INT8 variant exported alongside;
adopted only if parity holds.

**Phase 2 — `ciel-system1` binary.** New crate/binary (kept out of the
`ciel` hook binary so its dep weight — `ort`, `tokenizers`, an HTTP stack —
doesn't inflate the hot hook path): axum+tokio async server, `ort` sessions
with ordered EP registration (CUDA → CPU on Linux; CoreML on macOS;
DirectML on Windows), `is_available()` probing + graceful fallback, ORT
intra-op thread cap from physical cores, model residency map mirroring
`LAYA_MAX_LOADED`, same `env` file schema so one config drives both servers.
Prewarm becomes in-process (spawn warm calls at boot) — no external
warmup script needed on this path.

**Phase 3 — parity + cutover gate.** A `system1-parity` test corpus (fixed
state/question sets incl. edge shapes) diffed against `laya-serve` answers:
identical choices, |Δconfidence| ≤ 1e-3 (fp32) / agreed bound (INT8), plus
wire-shape conformance. The Rust server runs shadow-only until parity
holds; `ciel-system1.service` keeps laya-serve as fallback unit.

**Phase 4 — deploy.** `LAYA_BACKEND`/`CIEL_SYSTEM1_SERVER` switch in env;
documented rollback = restart the laya-serve unit.

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
  bound, or the INT8/ORT path does not ship.
- INT8 quantization only with per-checkpoint parity evidence; fp32 remains
  the default artifact.
- Key auth, remote redaction, `CIEL_SYSTEM1_DISABLED`, bounded timeouts,
  `LAYA_MAX_LOADED` residency, and `HF_HUB_OFFLINE`-equivalent offline
  operation all carry over unchanged.
- Artifact integrity: exported `.onnx` digests join the
  `LAYA_SHA256_DIGESTS`-style verification story; no unverified weights.
- New Rust deps (`ort`, `tokenizers`, `axum`/`tokio`) hash-pinned via
  Cargo.lock; `ort`'s binary runtime resolved without silent downloads
  (vendored or system ORT, documented).
- No publish/tag without the release gate: ~complete coverage of new code
  + Council sign-off on the release diff.

## Rollback

The ONNX artifacts and the new binary are additive: `ciel-system1.service`
continues to run laya-serve until the cutover flag flips, and rollback is
`systemctl --user restart ciel-system1` with the flag unset. The Python
fallback engine in hooks/lib is untouched.

## Open questions for the Council

1. INT8 adoption: is |Δconfidence| ≤ ~1e-2 acceptable per checkpoint with
   parity evidence, or should INT8 be gated to benchmarks-only status?
2. Server crate shape: standalone `ciel-system1` binary (recommended —
   keeps `ciel`'s dep closure lean) vs a cargo feature inside ciel-rs.
3. EP distribution on Linux: vendored ONNX Runtime via `ort`'s download
   vs `load-dynamic` against a system-installed libonnxruntime — which is
   acceptable under the no-silent-network mandate?
4. Does the Council require a shadow-window (e.g., N decisions dual-run)
   before cutover, or is corpus-parity sufficient?
5. Hosted Jev as an opt-in fallback tier (Alt D) alongside local Rust —
   in or out of this docket's scope?

## Implementation status (pre-verdict)

Research only. Findings verified: `export_onnx.py` present upstream,
`receptron/laya-onnx` reachable (english fp32 bundle), `ort` EP API
confirmed, `laya-serve` Router-only backend confirmed from source. No code
changes proposed beyond this docket.
