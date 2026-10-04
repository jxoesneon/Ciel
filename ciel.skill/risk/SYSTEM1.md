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
| Local laya-serve (default) | `CIEL_SYSTEM1_URL=http://127.0.0.1:8765` |
| Hosted Jev | `CIEL_SYSTEM1_URL=https://jev-agent.com` + `CIEL_SYSTEM1_KEY=<jv_live_...>` |
| AutoJev | `CIEL_SYSTEM1_URL=https://autojev.ai` + `CIEL_SYSTEM1_KEY=<key>` |

`CIEL_SYSTEM1_KEY` overrides; otherwise the key is read from
`~/.ciel/system1/env` (`LAYA_API_KEY=` line). `CIEL_SYSTEM1_DISABLED=1`
turns the tier off entirely. `CIEL_SYSTEM1_MODEL` (env or the same env
file) pins the request `model` — a laya checkpoint name
(`english`, `multilingual`, `typed-decisions`) or `jev-latest` on hosted
Jev.

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
routing: it shortlists high-cardinality candidate sets lexically
(`shortlist_options`, k=10) before the choice call — the documented
coarse-to-fine pattern once options exceed ~20.

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
| `pre_tool_risk` | 0.65 | PreToolUse safety gate |
| `router` | 0.82 | Skill routing confidence floor |
| `completion_check` | 0.75 | Completion verification |
| `council_prescreen` | 0.70 | Council event triage |

The `_band()` function in both Rust and Python calls `surface_tau(surface)` to
resolve the threshold, then classifies the confidence into `high` / `moderate` /
`low` bands relative to that per-surface τ.

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
~/.ciel/system1/venv/bin/pip install "laya[serve]"

# ~/.ciel/system1/env  (chmod 600 — contains the key; machine-local, never committed)
LAYA_HOST=127.0.0.1
LAYA_PORT=8765
LAYA_PRELOAD=1
LAYA_MODELS=typed-decisions         # single resident model (< 1.5 GB RSS; calibrated default)
LAYA_MAX_LOADED=1                   # cap resident models to 1 under Termux memory limits
LAYA_THREADS=4                      # torch intra-op CPU threads (2 or 4 for mobile cores)
LAYA_API_KEY=<openssl rand -hex 24>
CIEL_SYSTEM1_MODEL=typed-decisions    # pin the calibrated checkpoint for all asks
```

`~/.ciel/system1/serve.sh` sources `env` and execs `venv/bin/laya-serve`
(template: `init/system1/serve.sh`, env template: `init/system1/env.example`).
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
