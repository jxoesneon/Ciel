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
   (`scripts/verify_completion.py`, `skills/ciel/scripts/verify_evidence.sh`),
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
- **CLI Gate**: `python3 scripts/verify_completion.py --objective "..." --evidence "..." --task-class code_change --gate enforce`
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

## Local setup (laya)

```bash
python3 -m venv ~/.ciel/system1/venv
~/.ciel/system1/venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu
~/.ciel/system1/venv/bin/pip install "laya[serve]"

# ~/.ciel/system1/env  (chmod 600 — contains the key; machine-local, never committed)
LAYA_HOST=127.0.0.1
LAYA_PORT=8765
LAYA_PRELOAD=1
LAYA_MODELS=english,typed-decisions   # both resident (~1.6GB); typed-decisions is the calibrated default
LAYA_THREADS=4
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
 "cache_hit": false, "latency_ms": 1966}
```
