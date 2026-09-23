# SYSTEM1 — shadow semantic tier for the pre-tool gate

`hooks/lib/risk_policy.py` supports an optional second opinion from a
System-1 decision model speaking the Jev protocol (`POST /v1/systemone`):
typed `choice`/`noul`/`score` questions over a compact state, answered with
calibrated probabilities in a single forward pass. No text generation.

## Design: shadow first

The base checkpoints are near-chance zero-shot on this domain, so the tier is
**strictly advisory**: it never influences `evaluate()`'s decision. Each
pre-tool hook fires a detached `--shadow` subprocess (zero added latency —
CPU inference can take seconds) that appends the verdict to
`~/.ciel/system1/shadow.log`, correlated to `activity.log` by `ts`.

Promotion path: shadow → measure agreement on real traffic + the red-team
corpus → fine-tune a domain checkpoint (RLCD) → advisory tier →
confirm/deny escalation, each step gated by the Council.

## Backends

| Backend | Config |
| --- | --- |
| Local laya-serve (default) | `CIEL_SYSTEM1_URL=http://127.0.0.1:8765` |
| Hosted Jev | `CIEL_SYSTEM1_URL=https://jev-agent.com` + `CIEL_SYSTEM1_KEY=<jv_live_...>` |
| AutoJev | `CIEL_SYSTEM1_URL=https://autojev.ai` + `CIEL_SYSTEM1_KEY=<key>` |

`CIEL_SYSTEM1_KEY` overrides; otherwise the key is read from
`~/.ciel/system1/env` (`LAYA_API_KEY=` line). `CIEL_SYSTEM1_DISABLED=1`
turns the tier off entirely.

## Local setup (laya)

```bash
python3 -m venv ~/.ciel/system1/venv
~/.ciel/system1/venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu
~/.ciel/system1/venv/bin/pip install "laya[serve]"

# ~/.ciel/system1/env  (chmod 600 — contains the key; machine-local, never committed)
LAYA_HOST=127.0.0.1
LAYA_PORT=8765
LAYA_PRELOAD=1
LAYA_MODELS=english        # only the English checkpoint resident (~800MB RAM)
LAYA_THREADS=4
LAYA_API_KEY=<openssl rand -hex 24>
```

`~/.ciel/system1/serve.sh` sources `env` and execs `venv/bin/laya-serve`;
a systemd user unit (`ciel-system1.service`) keeps it resident.

## Record format (shadow.log)

```json
{"ts": "...", "runtime": "devin", "tool": "exec", "command": "...",
 "regex_decision": "allow", "rule_id": null,
 "system1": {"choice": "dangerous", "confidence": 0.41, "model": "english"}}
```
