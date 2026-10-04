# SYSTEM1 — Integrated Decision Tier (Laya / Jev Protocol)

`system1` is the shared client for a System-1 decision model speaking the
Jev protocol (`POST /v1/systemone`): typed `choice`/`noul`/`score` questions
over a compact state, answered with probabilities in a single forward pass.
No text generation.

## Integration: Active Pipeline Tier

System-1 is **fully integrated into the live execution pipeline**:
1. **PreToolUse Safety Gate**: Synchronously evaluates semantic risk in the
   hot path. If deterministic regex policy allows an operation but System-1
   detects destructive, privilege-escalating, or sensitive-path danger
   (`dangerous` choice with confidence $\ge \tau$), it **actively intercepts
   and blocks/denies** the operation (unless overridden by `allow_privileged`).
2. **Router**: Provides coarse-to-fine candidate ranking (`route_choice`)
   on ambiguous or multi-skill matches before LLM reasoning fallback.
3. **Completion Verification**: Powers `completion_check` surface to verify
   empirical evidence against deliverables. Intercepts false passes in
   `paired_eval.py` (`--completion-gate [shadow|enforce]`), post-task scripts
   (`ciel.skill/init/scripts/verify_completion.py`, `skills/ciel/scripts/verify_evidence.py`),
   and `system1.rs` (`evaluate_completion`) using typed choices and 1–5 rubric scores.
4. **Council Prescreen**: Triage for council-scoped events.

Operating modes via `CIEL_SYSTEM1_MODE`:
- `active` (default): Synchronous active evaluation in the pipeline with
  fail-open protection when offline.
- `shadow`: Detached background logging only (`events.jsonl`).
- `off`: Completely disabled.

## Backends

| Backend | Config |
| --- | --- |
| Local laya-serve (default) | `CIEL_SYSTEM1_URL=http://127.0.0.1:8765` + `LAYA_API_KEY` |
| Hosted Jev failover (primary when keyed) | `JEV_API_KEY=<apikey_...>` (+ optional `CIEL_SYSTEM1_HOSTED_URL`, default `https://api.typesafe.ai`) |
| Hosted Jev (official, sole engine) | `CIEL_SYSTEM1_URL=https://api.typesafe.ai` + `JEV_API_KEY=<apikey_...>` |
| Hosted Jev (unofficial proxy, sole engine) | `CIEL_SYSTEM1_URL=https://jev-agent.com` + `CIEL_SYSTEM1_KEY=<jv_live_...>` |
| AutoJev | `CIEL_SYSTEM1_URL=https://autojev.ai` + `CIEL_SYSTEM1_KEY=<key>` |

**Failover chain:** with a local `CIEL_SYSTEM1_URL` and a hosted key
configured (`CIEL_SYSTEM1_HOSTED_KEY` → `JEV_API_KEY` → `CIEL_SYSTEM1_KEY`,
process env or the env file), every ask tries hosted Jev first and falls
back to local laya with whatever timeout remains. A persisted circuit
breaker at `~/.ciel/system1/hosted_state.json` keeps laya primary until
the backoff expires — 401/402/403 (key/balance) trip 300s, 429 trips 60s,
transport/5xx trip 30s; request-shape 4xx fall through without tripping.
`CIEL_SYSTEM1_HOSTED=off` disables hosted egress entirely; no hosted key
means local-only. `LAYA_API_KEY` is never sent to a hosted endpoint, and
the local leg never receives the Jev `model` id.

`CIEL_SYSTEM1_URL` is a base URL: a bare jev-agent.com host resolves to
`…/api/v1/systemone`; every other base (including `api.typesafe.ai`)
resolves to `…/v1/systemone`, and a base that already contains the full
path is used verbatim. When the resolved endpoint is not loopback, the
outbound `state` payload is secret-redacted — the same redaction applied
to the local `events.jsonl` log — and hosted endpoints require `https`.

`CIEL_SYSTEM1_KEY` overrides; otherwise the key is read from
`~/.ciel/system1/env` — local endpoints use `CIEL_SYSTEM1_KEY`/
`LAYA_API_KEY`, hosted legs use the chain above.
`CIEL_SYSTEM1_DISABLED=1` turns the tier off entirely.
`CIEL_SYSTEM1_MODEL` (env or the same env file) pins the request `model`
for local laya checkpoints (`english`, `multilingual`, `typed-decisions`).
Hosted endpoints ignore it — the Jev API 400s on unknown model ids and
422s when `model` is absent — and always send `CIEL_SYSTEM1_HOSTED_MODEL`
(default `jev-latest`; valid ids per `GET /v1/models`: `jev-latest`,
`jev-preview`).

Hosted Jev exposes no `/batch` route, so `ask_batch()` serializes through
single asks when the endpoint is hosted — one billed request per state.
Hosted answers carry a `usage` token block and omit `confidence` on noul
answers (local laya reports `max(p, 1-p)` there); both shapes expose
`answers` + `model` identically to callers.

## Completion Verification Pipeline

System-1 evaluates objective deliverables vs. empirical evidence artifacts
(defined in `COMPLETION_EVIDENCE.md`) using typed questions:

1. **Typed Choice (`done`)**: `complete` ("objective is fully satisfied with direct empirical proof and verification artifacts") vs. `incomplete` ("objective is unverified, missing required artifacts, failed verification, or asserts claims without evidence"). Flagged choices (`incomplete`) trigger active gate intercepts in `enforce` mode.
2. **Typed Score (`evidence_score`)**: 1–5 quality rubric ranging from `1` (pure assertion/contradictory) to `5` (complete reproducible empirical verification of all task-class artifacts).

Invocation methods:
- **Python Library**: `system1.completion_check(objective, evidence, task_class, with_score=True)`
- **Rust Client**: `system1::evaluate_completion(objective, evidence, task_class, timeout_s)`
- **CLI Gate**: `python3 ciel.skill/init/scripts/verify_completion.py --objective "..." --evidence "..." --task-class code_change --gate enforce` (deployed to `~/.ciel/scripts/` on install; binary fast path: `ciel verify-completion`)
- **Eval Pipeline**: `python3 scripts/paired_eval.py --skill <dir> --completion-gate enforce`

All verification points adhere strictly to **fail-open semantics**: if the System-1 endpoint is unreachable, offline, times out, or disabled, the gate passes through the deterministic test runner verdict without blocking.

## On-demand asks (`--decide`)

Hooks only cover surfaces with an interception point. For deliberate
decisions — completion checks, task routing, salience — call the endpoint
synchronously; the verdict prints to stdout and still lands in
`events.jsonl`:

```bash
echo '{
  "surface": "completion_check",
  "state": {"objective": "...", "evidence": "..."},
  "questions": {"done": {"type": "choice",
    "instructions": "Is the objective verifiably complete?",
    "criteria": {"complete": "all claims verified",
                 "incomplete": "anything unverified or missing"}}}
}' | python3 ~/.ciel/hooks/lib/system1.py --decide
```

Prints `{"answers": {...}, "model": "typed-decisions"}` or `null` when the
endpoint is down (fail-open). Use `system1.route_choice(task, options)` for
routing: it shortlists high-cardinality candidate sets
(`shortlist_options`, k=10 — semantic top-k via the embed helper when
available, unioned with IDF-weighted lexical top-5 and exact name-token
matches) before the choice call — the documented coarse-to-fine pattern
once options exceed ~20. Set `CIEL_SYSTEM1_EMBED=0` to force lexical-only,
or `CIEL_SYSTEM1_EMBED_BIN` to swap in a native embedder.

## CLI Governance Commands

### `ciel route-choice`

Reads a JSON payload from stdin and outputs a `RouteVerdict` JSON:

```bash
echo '{"task": "deploy the container", "options": ["docker", "kubernetes", "podman"]}' \
  | ciel route-choice
```

Input fields: `task` (string), `options` (array), optional `k` (shortlist size,
default 10), optional `timeout` (seconds, default 5).

Output: `{"status": "ok", "degraded": false, "choice": "docker",
"confidence": 0.41, "margin": 0.17, "shortlist": ["docker","kubernetes","podman"],
"model": "typed-decisions"}`.

When System-1 is offline: `{"status": "ok", "degraded": true, "choice": "<first option>", ...}`.

### `ciel verify-completion`

Verifies that empirical evidence satisfies a stated objective:

```bash
ciel verify-completion \
  --objective "All 43 Rust tests pass" \
  --evidence "cargo test output: 43 passed; 0 failed" \
  --gate enforce
```

Or via stdin JSON: `{"objective": "...", "evidence": "...", "task_class": "code_change"}`.

Exit codes: `0` = pass, `2` = deny (enforce mode only), `1` = error.

Output includes `status`, `degraded`, `decision` (pass/deny/fail_open_pass),
`verified` (bool), `score` (1–5), `band`, `confidence`.

### `ciel system1 --surface <name>`

Override the evaluation surface for ad-hoc `--ask` / `--decide` invocations:

```bash
echo '{"state": {...}, "questions": {...}}' \
  | ciel system1 --surface completion_check --decide
```

This selects the per-surface threshold from the calibration lattice instead of
the default `pre_tool_risk` tau.

## Multi-Surface Threshold Lattice

Each evaluation surface has an independent confidence threshold (τ). The
threshold is resolved with **4-tier precedence** (highest wins):

1. **Environment per-surface**: `CIEL_SYSTEM1_TAU_PRE_TOOL_RISK=0.60`
2. **Environment global**: `CIEL_SYSTEM1_TAU=0.70`
3. **Policy config** (`policy.json` → `system1_thresholds`, or
   `system1_calibration.json` → `threshold_lattice`)
4. **Built-in defaults**:

| Surface | Default τ | Purpose |
|---------|-----------|---------|
| `pre_tool_risk` | 0.025 | PreToolUse safety gate |
| `router` | 0.47 | Skill routing confidence floor |
| `router_selection` | 0.33 | Shortlisted skill pick |
| `completion_check` | 0.10 | Completion verification |
| `council_prescreen` | 0.025 | Council event triage |
| `context_select` | 0.05 | Batched keep/drop per context item |
| `memory_salience` | 0.05 | Long-term-memory write-back gate |
| `context_compaction` | 0.02 | Semantic compaction trigger |
| `mandate_canary` | 0.15 | Operating-mandate drift check |

These defaults are calibrated for the `typed-decisions` checkpoint's
compressed confidence range (~0.0001–0.16) — each floor sits above every
observed wrong-direction confidence in the alignment corpus, so permissive
answers degrade to `uncertain` instead of silently passing. Recalibrate
against the live checkpoint before trusting them on a different model.

The `_band()` function in both Rust and Python calls `surface_tau(surface)` to
resolve the threshold, then classifies the confidence into `high` / `moderate` /
`low` bands relative to that per-surface τ.

## Context surfaces

Four advisory surfaces serve context management. All use two-option `choice`
in place of `noul` per ADR_20260923 (the base checkpoint's noul head follows
label wording, not state), and all are fail-open — `None` means "no verdict":
`context_select` callers must treat None as *keep everything*, and a `drop`
verdict only takes effect at or above τ.

- `context_select(task, candidates, k, timeout)` — coarse `shortlist_options`
  pre-filter, then one `POST …/batch` call giving each candidate its own
  keep/drop verdict (a multi-selector; `route_choice` picks a single winner).
  `CONTEXT_SELECT_BUDGET_S = 1.5` is the binding latency ceiling for
  latency-sensitive callers.
- `memory_salience(event)` — store/skip gate before a record enters long-term
  memory.
- `compaction_decision(stats)` — `{continue, compress, drop_stale, escalate}`
  choice plus a 5-level pressure `score`; non-`continue` actions raise the
  flag band.
- `mandate_canary(mandates, context)` — operative/drifted check on operating
  mandates in context; `drifted` raises the flag band.

`ask_batch(states, questions, timeout)` is the shared batch client
(`/v1/systemone/batch`, 64 states per request, chunked; positional
`{answers, model}` results, None per malformed item or on failure).

## Fail-Open Network Hardening

All HTTP calls to the System-1 endpoint use bounded connect timeouts to
prevent pipeline stalls:

- **Loopback** (127.0.0.1 / localhost): 10ms connect timeout
- **Remote**: 50ms connect timeout
- **curl fallback**: `--connect-timeout 0.05`

When the endpoint is unreachable, the tier returns `None` (Rust) or `null`
(Python), the `degraded` flag is set, and the pipeline proceeds — hard deny
rules in `risk.rs` are never affected by System-1 availability.

## Local setup (laya)

```bash
python3 -m venv ~/.ciel/system1/venv
~/.ciel/system1/venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu
# hash-pinned install — init/system1/requirements-laya.txt carries the full
# resolved set for laya[serve]==0.3.26 with PyPI/pytorch-index sha256 digests:
~/.ciel/system1/venv/bin/pip install --require-hashes -r init/system1/requirements-laya.txt

# ~/.ciel/system1/env  (chmod 600 — contains the key; machine-local, never committed)
LAYA_HOST=127.0.0.1
LAYA_PORT=8765
LAYA_PRELOAD=1
LAYA_MODELS=english,typed-decisions # keep resident what routing may choose
LAYA_MAX_LOADED=2                   # < routing set ⇒ ~7s CPU reload per switch
LAYA_THREADS=2                      # cap at PHYSICAL cores (oversubscription regresses)
LAYA_AUTO_TASK=1                    # unpinned calls route to typed-decisions, not the 0.36-acc base
LAYA_REVISION=<snapshot sha>        # supply-chain pin: freeze the Hub revision…
LAYA_SHA256_DIGESTS='<per-checkpoint JSON>'  # …and verify every artifact's sha256 at load
LAYA_API_KEY=<openssl rand -hex 24>
CIEL_SYSTEM1_MODEL=typed-decisions    # pin the calibrated checkpoint for all asks
```

`~/.ciel/system1/serve.sh` sources `env` and execs `venv/bin/laya-serve`
(template: `init/system1/serve.sh`, env template: `init/system1/env.example`).

**Prewarm.** The one-time first-forward JIT cost (~8s per checkpoint on
low-core CPU) is paid at two triggers so the first real request is never a
stall:

- service restart — detached `ExecStartPost` runs `warmup.sh`
- session start — `ciel session-start` spawns a detached
  `ciel system1 --warmup`; the shell fallback hooks exec `warmup.sh`
  instead. Both fail open and never block session start.

`warmup.sh` prefers the Rust client (`ciel system1 --warmup`, which reads
`LAYA_MODELS` and retries one tiny question per checkpoint until the
deadline); a curl loop is the dependency-free fallback.

**Device selection.** `LAYA_DEVICE` unset auto-detects cuda → mps → xpu →
cpu with silent per-checkpoint GPU→CPU fallback — leave it unset unless
pinning a box. The laya SDK ships `torch.compile`, ONNX, and TileLang+CUDA
backends (`backend="compile"|"onnx"|"tilelang"`), but `laya-serve` does not
expose them — it always instantiates the PyTorch `Router`. Using those
backends would need a custom serve shim (a heavier trade: new runtime
deps, exported artifacts, behavioral-parity burden).

Two supply-chain layers, both recommended: `requirements-laya.txt`
(`--require-hashes`, package level) and `LAYA_REVISION` +
`LAYA_SHA256_DIGESTS` (model-artifact level — tampered weights refuse to
load). Note that a pinned `laya-serve` can still make outbound HuggingFace
pulls when adding or updating checkpoints (`HF_HUB_OFFLINE=1` disables that
once the cache is warm).

**Measured CPU latency** (2-core/4-thread x86, warm): ~0.7–1.0 s per
decision, ~0.6 s per state in a `/batch` forward pass; first-forward JIT is
~8 s once per checkpoint (eliminated by the warmup). Budget accordingly:
the context surfaces fit async/context-build paths on modest CPU; keep
`CONTEXT_SELECT_BUDGET_S` enforced and treat `None` as keep-all. Do not
substitute `english` for `typed-decisions` to chase latency — the base
checkpoint scores 0.362 on the typed-decisions benchmark vs 0.766
fine-tuned.

The tier is meant to be on by default, so the supervisor must keep it
resident and self-healing — platform specifics:

| Platform | Supervisor | Unit (repo template) | Self-heal |
| --- | --- | --- | --- |
| Linux | systemd user unit | `init/systemd/ciel-system1.service` → `~/.config/systemd/user/` | `Restart=always` |
| macOS | launchd agent | `init/launchd/com.ciel.system1.plist` (`__HOME__` token) → `~/Library/LaunchAgents/` | `KeepAlive=true` + `RunAtLoad=true` + `ThrottleInterval=10` |

macOS install (verified):

```bash
plutil -lint ~/Library/LaunchAgents/com.ciel.system1.plist
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.ciel.system1.plist
launchctl print gui/$(id -u)/com.ciel.system1   # state = running
```

The plist runs `serve.sh` directly, logs to `~/.ciel/system1/serve.{out,err}.log`,
and respawns the server within `ThrottleInterval` seconds of any exit. If the
HF hub rate-limits unauthenticated model pulls, add `HF_TOKEN=` to `env`.

macOS note: `LAYA_DEVICE` auto-selects `mps` (Apple GPU) when available —
checkpoints do not stay on CPU. `/health` reports `loaded`, `revisions`,
`device`, and per-checkpoint `cpu_fallbacks`.

## Record format (events.jsonl)

```json
{"ts": "...", "surface": "pre_tool_risk", "questions": {...},
 "meta": {"ts": "...", "runtime": "devin", "regex_decision": "allow",
          "rule_id": null},
 "system1": {"answers": {"risk": {"choice": "dangerous",
             "confidence": 0.41}}, "model": "english"},
 "cache_hit": false, "latency_ms": 1966, "degraded": false}
```

Completion verification events include `score` and `decision`:

```json
{"ts": "...", "surface": "completion_check",
 "system1": {"answers": {"done": {"choice": "complete", "confidence": 0.88},
             "evidence_score": {"score": 5, "confidence": 0.76}},
             "model": "typed-decisions"},
 "decision": "pass", "degraded": false, "latency_ms": 342}
```
