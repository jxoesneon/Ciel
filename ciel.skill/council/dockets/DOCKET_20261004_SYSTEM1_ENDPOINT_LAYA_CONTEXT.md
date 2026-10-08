# DOCKET 2026-10-04 — System-1 Hosted-Endpoint Normalization, Laya 0.3.26, and Context-Management Surfaces

- **Scope:** SELF_MODIFICATION (non-locked; touches `hooks/lib/system1.py`, `ciel-rs/src/system1.rs`, and proposes new lattice surfaces)
- **Thresholds:** pass_weighted_score 7.5, safety_min 7, majority_required 4/5 at pass_score 6
- **Repo:** `/home/eduardo/Ciel` (endpoint fix implemented + verified, uncommitted; awaiting Council verdict on forward surfaces)
- **Host env:** `~/.ciel/system1/venv` upgraded `laya 0.3.10 → 0.3.26`; `ciel-system1.service` (systemd user) live-verified

## Background

Two defects/gaps found during the Jev/Laya context-management research pass:

1. **Hosted-endpoint path bug.** `hooks/lib/system1.py::ask()` and
   `ciel-rs/src/system1.rs::ask()` unconditionally appended `/v1/systemone` to
   `CIEL_SYSTEM1_URL`. Hosted Jev serves `…/api/v1/systemone`; the documented
   config `CIEL_SYSTEM1_URL=https://jev-agent.com` therefore 404s on every call —
   System-1 was silently unreachable on any host configured per the docs.
2. **Laya 0.3.10 predates `/v1/systemone/batch`.** Batch (≤64 states, shared
   question set) landed in 0.3.22; without it, per-candidate relevance decisions
   cost one HTTP round-trip each, which is prohibitive on pre-tool paths.

## Evidence (verified)

| Check | Result |
| --- | --- |
| `endpoint()` normalization, Python + Rust | identical semantics in both engines |
| `https://jev-agent.com{,/,/:443,/api}` | → `…/api/v1/systemone` (no double `/api`, port preserved) |
| `http://127.0.0.1:8765` → `/v1/systemone` | local/OSS bases unchanged |
| explicit `…/v1/systemone` | passes through verbatim |
| `tests/test_system1.py` (incl. new endpoint cases) | OK |
| `cargo test` (System-1 suite) | 44/44, incl. `endpoint_normalizes_hosted_and_local` |
| `cargo fmt --check`, `clippy`, `cargo build --release` | clean; binary + lib deployed to `~/.ciel` |
| laya upgrade | `pip show laya` → 0.3.26 (`laya[serve]`) |
| service health | `GET /health` → `{"status":"ok"}` (systemd user unit, auto-restart) |
| live `POST /v1/systemone` | typed `choice` answer returned (`laya-rl-agent`, router→`english`) |
| live `POST /v1/systemone/batch` | 2-state batch returns per-state results — **batch confirmed working** |

Incidental repair: a prior permission sweep had flattened `~/.ciel/system1/**`
(incl. `serve.sh` and every `venv/bin` entry point) to `0600`, crash-looping the
service (`203/EXEC`). Exec bits restored; unit is `active (running)`.

## Proposal under review (forward work — requires Council verdict)

Use the now-batch-capable System-1 as a typed decision layer over context, per
`router/CONTEXT_BUDGET.md` (L0→L1→L2 progressive disclosure):

1. **`context_select` lattice surface** — embeddings/lexical first-pass shortlist
   (existing `system1_embed` + `shortlist_options`), then one batched `noul`
   question per candidate ("relevant to the current task?") as a multi-selector.
   ~τ 0.60; one batch call vs N serial asks.
2. **Salience gate on write-back** — `noul` + `score` on session events before
   anything enters MemPalace/context files; blocks memory bloat at the source.
3. **Compaction trigger** — `score` 1–5 + `choice`{continue, compress,
   drop_stale, escalate} at prompt-submit boundaries; semantic pressure
   detection feeding `context_summarizer` rather than token-count heuristics.
4. **Canary/mandate detector** — `noul` "operating mandates still operative in
   context?", converting the convention into a measured check.
5. **`ciel-context` checkpoint** — RLCD fine-tune on `events.jsonl` (1,712
   decisions) + `rlcd_pairs.jsonl` (73 pairs) once decision data accrues.

Runtime-safety constraints (binding on any implementation):

- bounded latency budget on the pre-tool path; batch preferred, serial asks
  capped; fail-open/failsafe behavior unchanged where already specified;
- deterministic regex failsafe remains authoritative for destructive ops —
  System-1 may only *tighten*, never loosen, a rule verdict;
- cache semantics preserved (SHA-256 state/questions key).

## Alternatives the Council may weigh

- **A. Endpoint fix only** — ship normalization + laya bump; defer all surfaces.
- **B. Proposal as written** — fix + surfaces in staged order (1→5).
- **C. Surfaces without fine-tune** — implement 1–4, hold the checkpoint until a
  dedicated data-quality docket.
- **D. Embedding-only selection** — skip typed decisions; rely on bi-encoder
  shortlist alone (cheaper, no calibrated relevance).

## Rollback

- Endpoint normalization is additive and URL-shape-only; revert = single commit
  per engine. Local/OSS URL forms are bit-identical to prior behavior.
- Laya rollback: `pip install "laya[serve]==0.3.10"`; `env`, model cache, and
  unit file are untouched by the upgrade. No schema/migration steps observed.

## Open questions for the Council

- Latency ceiling for batch `context_select` on the pre-tool path (ms budget)?
- Should salience gating apply to `events.jsonl` writes too, or only
  MemPalace/context-file writes?
- Is a new lattice surface (`context_select`) preferred over extending the
  existing risk surfaces' question sets?
- Coverage target for the new surfaces before publish (per the release gate)?

## Verdict (RUN_20261004_SYSTEM1_LAYA_CONTEXT)

**PASS — weighted 8.0** (self-modification thresholds: ≥7.5 weighted, Safety ≥7,
≥4/5 at pass_score 6)

| Lens | Stage 1 | Stage 2 | Delta |
| --- | --- | --- | --- |
| Coherence | 8 | 8 | 0 |
| Capability | 8 | 8 | 0 |
| Safety | 8 | 8 | 0 (veto: false) |
| Efficiency | 8 | 8 | 0 |
| Evolution | 8 | 8 | 0 |

Pivotal lens: **safety** — its Stage-2 finding that `_redact()` covered only the
local `events.jsonl` append, never the outbound wire body, produced the most
consequential binding amendment (implemented in both engines before verdict
recording).

Stage-2 challenges adjudicated: Capability/Coherence→Efficiency refuted
`duplicates:router_selection` on the code (a `choice` question yields one argmax
winner; `context_select` is a batched per-candidate `noul` multi-selector — a
shape `choice` cannot express). Efficiency→Safety was technically correct on
specifics (a schemeless `jev-agent.com` does match; a userinfo authority does
pass the host check) — the userinfo edge was hardened in both engines.
Evolution→Capability corrected "salience gate is entirely new": `memory_salience`
was already ratified in ADR_20260923 — the docket executes scheduled work.

Binding amendments: `~/.ciel/council/RUN_20261004_SYSTEM1_LAYA_CONTEXT/final_verdict.json`.
Headline items: outbound state is secret-redacted whenever the endpoint is
non-loopback; Option C adopted (RLCD checkpoint deferred to a dedicated
data-quality docket); a concrete ms latency ceiling is required before
`context_select` lands on the pre-tool path; `context_select` is confirmed a
new surface; a `batch_decide()` client with tests precedes any surface
consumption; laya installs are to be hash-pinned.

## Implementation status (post-verdict)

- Endpoint normalization + `HOSTED_API_HOSTS` const parity, userinfo-authority
  hardening, and `_remote()`/`remote()` egress redaction: implemented in both
  engines; `SYSTEM1.md` documents base-URL resolution and remote redaction.
- laya `0.3.10 → 0.3.26` verified live (`/health` ok; `/v1/systemone` and
  `/v1/systemone/batch` answered). Exec-bit repair on `~/.ciel/system1` noted.
- Tests: `tests/test_system1.py` 38/38 incl. wire-level redaction and
  schemeless/userinfo cases; Rust System-1 suite green (one pre-existing
  latency flake passes in isolation); fmt/clippy/release build clean.
- Forward surfaces 1–4 approved for staged implementation under the binding
  constraints; the `ciel-context` checkpoint is deferred to a dedicated
  data-quality docket (Option C).
