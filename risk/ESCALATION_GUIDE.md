# Escalation Guide

Ciel's risk classification and escalation system. Every operation is classified
before execution; classification drives gating. Constitutional floor: critical-
level operations always escalate to the user, regardless of project profile.

Source specs:
- `ciel.skill/risk/RISK.md` — model master
- `ciel.skill/risk/CLASSIFICATION.md` — criteria and examples (locked)
- `ciel.skill/risk/ESCALATION_LADDER.md` — end-to-end decision flow
- `ciel.skill/risk/LLM_JUDGE.md` — judge protocol
- `ciel.skill/risk/CRITICAL_RISK.md` — critical policy (locked)
- `ciel.skill/configuration/global/risk.config.md` — thresholds
- `ciel.skill/configuration/local/escalation.config.md` — project profile

State: `~/.ciel/risk/RISK_STATE.json`

---

## 1. Classification Levels

Every operation is scored 0–10 on six axes and combined via weighted max:

| Axis | Weight | Meaning |
| --- | --- | --- |
| reversibility | 0.25 | Can we undo this? |
| blast_radius | 0.20 | Worst case if it fails |
| external_impact | 0.20 | Touches systems outside local dev env |
| data_sensitivity | 0.15 | Secrets, PII, PHI, PCI |
| cost | 0.10 | Monetary or compute cost |
| novelty | 0.10 | Has Ciel done this successfully before |

`composite = Σ weight_i * axis_i`

| Level | Composite Range | Authority | Gate |
| --- | --- | --- | --- |
| low | < 3.0 | autonomous | log only |
| mid | 3.0 – < 6.0 | autonomous with judge | LLM judge before proceeding |
| high | 6.0 – < 8.5 | council-gated | full Council of Five |
| critical | ≥ 8.5 OR veto match | user escalation | user must approve, even after Council |

Any veto-condition match forces `level = critical` regardless of composite.

---

## 2. Four-Level Escalation Ladder

| Level | Condition | Action |
| --- | --- | --- |
| **A** | low_risk + registry_hit + confidence ≥ 0.8 | autonomous |
| **B** | low_risk + reasoning_path + confidence ≥ 0.7 | autonomous_with_log |
| **C** | mid/high_risk OR self_modification OR skill_acquisition | council_gated |
| **D** | critical_risk OR confidence < threshold OR council_deadlock | user_escalation |

- **A — Autonomous:** No judge, no Council. Execute and log.
- **B — Autonomous + Log:** Execute autonomously but write a structured
  reasoning envelope to `activity.log`. Used when there is a reasoning path
  but no registry hit.
- **C — Council-gated:** Full Council of Five invocation. Safety member is
  primary. Execute only on Council pass + Safety non-veto. Covers mid/high
  risk, self-modification, and skill acquisition.
- **D — User escalation:** Block execution until the user explicitly approves.
  Always used for critical risk, sub-threshold confidence, or Council deadlock.

---

## 3. Three-Step Escalation Sequence

When an operation is classified ≥ mid and research-first is enabled (default),
the escalation sequence is:

```
1. Deep Research  →  2. Council  →  3. User
```

### Step 1 — Deep Research
If `autonomy.research_first: true` (default), any classification ≥ mid receives
a research pass before deciding the gate. Research output (via
`seed_skills/research`) informs the judge / Council. Research is not a gate by
itself — it feeds the next step.

### Step 2 — Council (mid → judge, high → Council of Five)
- **Mid risk:** LLM judge (lightweight single-model self-audit, see §4). On
  `proceed` → execute. On `abort` → upgrade to high-risk Council treatment.
- **High risk:** Full Council of Five invocation. Safety member is primary.
  Execute only on Council pass + Safety non-veto. Council `deadlock` →
  escalate to user (Step 3).

### Step 3 — User Escalation
- Critical risk: always. No config value downgrades critical.
- Council deadlock: per `council/ESCALATION.md`.
- Sub-threshold confidence: `confidence < judge_confidence_floor` (0.70).

Construct a user escalation envelope containing:
- operation + rendered command,
- evidence trail of how we arrived here,
- risk composite score + axis breakdown,
- dry-run preview if available,
- reversibility analysis,
- proposed alternative (always one lower-risk alternative if feasible).

Block execution until user explicitly approves. User approval is recorded with
timestamp, operator id, and (if remote) device binding.

### Fall-Through Rules
- Judge `abort` → high-risk Council treatment.
- Council `deadlock` → `council/ESCALATION.md` → user escalation.
- User no-response within `escalation.timeout_hours` (default 72) → operation
  abandoned; Ciel logs and moves on.

---

## 4. LLM Judge — When to Use

The LLM judge is a lightweight self-audit for **mid-risk** operations only.
Ciel audits Ciel — a single-model prompt, not the full Council.

### When Called
- `risk/CLASSIFICATION.md` returns `mid`.
- `router/ROUTER.md` composition has any `mid` sub-step.
- Sensitive pre-execution gate in adapter hooks (Claude Code `PreToolUse`,
  Gemini CLI `tool.preinvoke`, generic inline prompt check).

### When NOT Called
- Low risk → log only, no judge.
- High risk → go straight to Council of Five.
- Critical risk → go straight to user escalation.

### Output Contract
```json
{
  "decision": "proceed | revise | abort",
  "confidence": 0.0..1.0,
  "concerns": ["..."],
  "mitigations": ["..."],
  "alternative_plan": "... | null"
}
```

### Semantics
- `proceed` — clear to execute. Requires `confidence ≥ judge_confidence_floor`
  (0.70).
- `revise` — execute after applying the listed mitigations.
- `abort` — upgrade to high-risk Council or escalate, depending on residual
  risk.

### Confidence Floor
`risk.config.judge_confidence_floor` = **0.70**. Below the floor → upgrade
regardless of `decision`.

### Sandbox-Pair (Preferred)
For mid-risk operations supported by dry-run, run the dry-run first, feed
output to the judge, then proceed on `proceed`. Dry-run + judge is preferred
over judge alone. Examples: `git push --dry-run`, `npm publish --dry-run`,
`terraform plan`, Gemini Plan mode.

### Model
Default: cheapest capable model (Haiku / Gemini Flash). For mid-risk with high
cost axis: upgrade to stronger model. Config: `mid_judge_model: auto`.

---

## 5. Critical Risk Policy

`locked: true`. Critical-risk operations **always** require user approval. No
config value downgrades them.

### Examples
- Destructive filesystem operations outside sandbox (`rm -rf` outside sandbox).
- Force-push / history rewrite on a shared branch (`git push --force`).
- Package publication to public registries (`npm publish`).
- External state-mutating API calls (email send, payment, SMS).
- Production database migration / destructive query.
- Infrastructure-as-code apply affecting prod.
- Modifying OS or system services.
- Activating computer use with destructive intent.

### Flow
1. Classify operation → critical.
2. Even if Council is convened, record its analysis but do not pre-authorize.
3. Construct user escalation envelope (see §3, Step 3).
4. Block execution until user explicitly approves.
5. Record approval with timestamp, operator id, device binding.

### Post-Execution
- Mandatory watch for at least `critical.watch_hours` (default **24**).
- Mandatory post-mortem written to `~/.ciel/high_risk/<run_id>.md` whether
  success or failure.
- Outcome feeds self-improvement; Ciel prefers to acquire a scripted-and-safer
  equivalent for future use.

### Forbidden
- Auto-approving on prior pattern ("you did this last week" is not consent).
- Delegating critical approval to another agent.
- Running critical ops headless without explicit approval flag.
- `accept_remote_approval: false` is locked — critical approvals must be
  in-terminal.

---

## 6. Safety Short-Circuit

**Destructive actions bypass the ladder and go directly to user escalation.**

Any operation matching a veto condition is forced to `level = critical`
regardless of composite score, which means direct user escalation (Level D).
Veto conditions include:
- irreversible + external_impact ≥ 8,
- data_sensitivity ≥ 9 without explicit user consent,
- license boundary violation,
- any item in `council/rubrics/VETO_CONDITIONS.md`.

The `safety_veto_floor` = **3**: Safety Council member veto cannot be
overridden by fewer than 3 concurring members. This is the floor below which
a Safety veto is absolute.

In Devin specifically, destructive operations (e.g. `rm -rf` outside sandbox,
force-push to shared branch, production DB writes) should short-circuit
directly to `ask_user_question` without passing through judge or Council.

---

## 7. User Escalation in Devin

When the escalation level is **D** (user escalation), use the
`ask_user_question` mechanism to obtain explicit user approval.

### How to Use
Present the user escalation envelope as a question with the operation, the
rendered command, the risk composite + axis breakdown, reversibility analysis,
dry-run preview (if available), and the proposed lower-risk alternative.

Ask a clear yes/no (or choose-between-alternatives) question. Do not execute
until approval is received.

### Recording
On approval, record in `activity.log`:
- timestamp,
- operator id,
- device binding (if remote — note: `accept_remote_approval: false` for
  critical, so critical approvals must be in-terminal),
- the escalation envelope reference,
- the run_id.

On rejection or no-response within `escalation.timeout_hours` (72h), abandon
the operation and log it.

---

## 8. Project Profile Shift

`configuration/local/escalation.config.md` (`effective = override ?? auto_detected`):

| Profile | Behavior |
| --- | --- |
| `research` | most permissive; only critical escalates |
| `development` | default; mid/high Council-gated |
| `production` | conservative; high+ always escalates |
| `regulated` | most restrictive; all mid+ escalates |

Constitutional floor: critical always escalates regardless of profile.
Override cannot be more permissive than `auto_detected` by more than one
category step, and cannot go below `research`.

Set via `/ciel override set <category>` or edit
`configuration/local/escalation.config.md`; Ciel reloads on next op.
