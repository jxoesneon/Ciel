# SYSTEM1 — shadow semantic tier for the pre-tool gate

`hooks/lib/system1.py` is the shared client for a System-1 decision model
speaking the Jev protocol (`POST /v1/systemone`): typed `choice`/`noul`/
`score` questions over a compact state, answered with probabilities in a
single forward pass. No text generation. `hooks/lib/risk_policy.py` is its
first consumer; see `architecture/ADR_20260923_SYSTEM1_DEEP_INTEGRATION.md`
for the multi-surface lattice design.

## Design: shadow first

The base checkpoints are near-chance zero-shot on this domain, so the tier is
**strictly advisory**: it never influences `evaluate()`'s decision. Each
pre-tool hook fires a detached `system1.py --ask` subprocess (zero added
latency — CPU inference can take seconds; bounded to 2 in-flight with a
response cache) that appends the verdict to
`~/.ciel/system1/events.jsonl`, correlated to `activity.log` by `meta.ts`.

Promotion path: shadow → `scripts/system1_eval.py` produces
`risk/system1_calibration.json` → fine-tune a domain checkpoint (RLCD) →
advisory tier → confirm escalation, each step gated by the Council.

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
```

`~/.ciel/system1/serve.sh` sources `env` and execs `venv/bin/laya-serve`;
a systemd user unit (`ciel-system1.service`) keeps it resident.

## Record format (events.jsonl)

```json
{"ts": "...", "surface": "pre_tool_risk", "questions": {...},
 "meta": {"ts": "...", "runtime": "devin", "regex_decision": "allow",
          "rule_id": null},
 "system1": {"answers": {"risk": {"choice": "dangerous",
             "confidence": 0.41}}, "model": "english"},
 "cache_hit": false, "latency_ms": 1966}
```
