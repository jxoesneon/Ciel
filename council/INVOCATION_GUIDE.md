# Council Invocation Guide

Operational reference for convening the Ciel Council of Five (Architecture) and Design Council of Five.

---

## 1. When to Invoke

### Architecture Council

Invoke the Architecture Council when the decision concerns **system integration, self-modification, or operational risk**. The five members are Coherence, Capability, Safety (veto), Efficiency, and Evolution.

**Invocation Scopes** (see `council/invocation_scopes/`):

| Scope | Trigger | Threshold |
| --- | --- | --- |
| `SELF_MODIFICATION` | Ciel modifies her own core config, router, council rubrics, constitution, or locked material | weighted ≥ 7.5, Safety ≥ 7, majority 4/5 |
| `SKILL_INTEGRATION` | An acquired skill candidate needs to join the registry | weighted ≥ 6.5, Safety > 3, majority 3/5 |
| `SKILL_CONFLICT` | Registry overlap or drift detected between registered skills | per `CONFLICT_RUBRIC.md` |
| `PROMOTION` | Promoting a local learning to global `~/.ciel/` | weighted ≥ 7.0, Safety + Efficiency both ≥ pass_score |
| `HIGH_RISK_OPS` | Irreversible/destructive operation requested | weighted ≥ 7.0, Safety ≥ 6, explicit user consent for irreversible |

### Design Council

Invoke the Design Council when the decision concerns **user-interface or user-experience artifacts**. The five members are Clarity, Inclusion (veto), Efficiency, Aesthetics, and Actionability.

**When to use Design Council instead of Architecture Council:**
- Evaluating a UI screen, component, or interaction flow
- Reviewing information architecture, accessibility, or visual design
- Assessing conversion funnels, affordances, or user task flows
- Any artifact where the primary question is "how does a human experience this?"

**Joint sessions** (both councils) are used when an artifact spans both system integration and UI/UX — e.g., a skill that includes UI standards (see `DOCKET_20260730_UI_UX_COUNCIL_SYNTHESIS.md`).

---

## 2. The Three-Stage Process

### Stage 1 — Independent Scoring (Parallel)

Each of the 5 members independently evaluates the candidate artifact against their lens.

**Inputs per member:**
- `artifact` — L1 representation of the candidate (L2 on request via `requests: ["L2"]`)
- `rubric` — `council/rubrics/SCORING.md` summary
- `neighbors` (Architecture) or `design_context` (Design) — related entries for comparison
- Scope-specific preamble from `council/invocation_scopes/<SCOPE>.md`

**Output per member:**
```json
{
  "member": "<lens_name>",
  "stage": 1,
  "score": 0,
  "rationale": "<=3-4 sentences with concrete citations",
  "flags": ["<flag_name>"],
  "requests": ["L2"],
  "veto": false
}
```

- Scores are 0–10 per `rubrics/SCORING.md`.
- Safety (Architecture) sets `"veto": true` only when score ≤ 3.
- Inclusion (Design) sets `"veto": true` only when score ≤ 3.
- Scores without rationale are discarded (treated as abstention).

### Stage 2 — Cross-Review (Anonymous)

Each member sees the **four peer outputs without attribution** and may revise their score.

**Anonymization protocol** (see `council/ANONYMIZATION.md`):
1. Chairman collects all 5 Stage 1 outputs.
2. Chairman assigns each an anonymous id from `{A, B, C, D, E}` — deterministic per run, scrambled across runs (seeded by `run_id`).
3. Chairman constructs a Stage 2 input bundle for each member containing **only the other four** anonymized outputs.
4. Chairman post-processes Stage 1 rationales to normalize stylistic tics (e.g., Safety's tendency to say "risk vectors") without changing meaning — reduces de-anonymization leakage.
5. Prompts (`prompts/council/<lens>_stage2.md`) explicitly instruct members not to attempt de-anonymization.

**Output per member:**
```json
{
  "member": "<lens_name>",
  "stage": 2,
  "score": 0,
  "delta": 0,
  "rationale": "<=3 sentences on why held/moved",
  "challenge_of": "A|B|C|D|E|null",
  "challenge_note": "string|null",
  "flags": ["..."],
  "veto": false
}
```

- Members revise **only** on arguments specific to their lens.
- The anonymization mapping is logged in `~/.ciel/council/<run_id>/mapping.json` for post-hoc audit but never shared during the run.
- `anonymize_stage2` is Constitutionally locked to `true` and cannot be disabled.

### Stage 3 — Chairman Synthesis

Ciel (as Chairman) synthesizes the five Stage 2 votes into a final verdict.

**Synthesis algorithm** (see `council/CHAIRMAN.md`):

```
weighted_score =
    0.20 * coherence
  + 0.20 * capability
  + 0.25 * safety          # heavier weight
  + 0.15 * efficiency
  + 0.20 * evolution

if safety <= 3: verdict = reject; return
if weighted_score >= weighted_pass AND majority >= 3 of 5 score >= pass_score: verdict = pass
elif weighted_score < reject_threshold: verdict = reject
else: verdict = deadlock
```

**Thresholds** (from `council.config.md`):
- `pass_score`: 6 (individual member)
- `weighted_pass`: 6.5 (chairman weighted)
- `reject_threshold`: 4.5
- `majority_required`: 3 of 5

**Design Council equivalent:**
- Inclusion ≤ 3 → reject (absolute veto, mirrors Safety)
- Same weighted formula with Inclusion replacing Safety's veto role
- `inclusion_veto_floor`: 3

**Output:**
```json
{
  "verdict": "pass|reject|deadlock",
  "weighted_score": 7.6,
  "pivotal_lens": "<lens_with_largest_deviation>",
  "votes": { "coherence": 7, "capability": 9, "safety": 7, "efficiency": 6, "evolution": 8 },
  "chairman_summary": "<=5 sentences synthesizing the decision",
  "next_action": "register|reject|escalate_user|rerun_council|user_constitutional_confirm",
  "mitigations_required": ["..."],
  "commit_sha": "abc123..."
}
```

Serialized to `~/.ciel/council/<run_id>.json` and mirrored into MemPalace partition `ciel/council/`.

---

## 3. Dispatching 5 Parallel Subagents for Stage 1

Stage 1 requires **5 isolated subagent invocations** — one per member lens. Each subagent runs independently with no visibility into the others' work.

**Dispatch pattern:**

```
For each member in [Coherence, Capability, Safety, Efficiency, Evolution]:
  spawn subagent (is_background: true) with:
    - persona: council/members/<MEMBER>.md
    - prompt: prompts/council/<member>_stage1.md
    - inputs: { artifact, rubric, neighbors, scope_preamble }
    - workspace isolation (git worktree or path locking) to prevent file collisions
```

**Key rules:**
- All 5 subagents are dispatched **concurrently** (not sequentially).
- Each subagent sees only its own persona file, the artifact, the rubric, and the scope preamble — never the other members' outputs.
- Subagents must return strict JSON per the output contract.
- A subagent timeout (default 60s per `stage_timeout_s`) counts as abstention.
- Minimum 3 non-abstaining members for quorum; Safety/Inclusion must be present and non-abstaining.

**After Stage 1 completes:**
- Chairman collects all 5 outputs.
- If any flags require inter-stage fixes (e.g., Safety compliance overstatement), Chairman may apply fixes before Stage 2 (see `DOCKET_20260730_UI_UX_COUNCIL_SYNTHESIS.md` for precedent).

**Stage 2 dispatch** is identical — 5 parallel subagents, each with their `prompts/council/<member>_stage2.md` prompt, the artifact, their own Stage 1 output, and the 4 anonymized peer votes.

---

## 4. How Anonymization Works (Stage 2)

**Purpose:** Prevent inter-member bias. Without anonymization, members over-weight Safety's opinion by pattern, or reflexively disagree with Efficiency if they stereotype the role as "cut things." Anonymization forces each vote to stand on rationale alone.

**Protocol** (see `council/ANONYMIZATION.md`):

1. **Assign IDs:** Chairman assigns each Stage 1 output an anonymous id from `{A, B, C, D, E}`. Mapping is deterministic per run (seeded by `run_id`) but scrambled across runs.
2. **Construct bundles:** For each member, Chairman builds a bundle containing **only the other four** anonymized outputs — never their own, never the mapping.
3. **Style normalization:** Chairman pre-passes Stage 1 rationales to normalize stylistic tics (e.g., Safety's "risk vectors" phrasing) without changing meaning. This reduces fingerprint leakage. The `prompts/council/chairman_synthesis.md` prompt has a pre-pass sub-prompt for this.
4. **Anti-de-anonymization instruction:** Each Stage 2 prompt explicitly instructs the member not to attempt de-anonymization and to operate purely on content.
5. **Audit trail:** The anonymization mapping is logged in `~/.ciel/council/<run_id>/mapping.json` but not shared with members during the run. Accessible post-hoc for audit.
6. **Residual leakage is acceptable:** Perfect anonymization is not the goal — mitigation is. Style fingerprints can leak identity, but the normalization pre-pass minimizes this.

**Disabling:** `anonymize_stage2` is Constitutionally locked to `true`. Any run with anonymization disabled (research/debug only) is flagged in `activity.log`.

---

## 5. Chairman Synthesis (Stage 3)

The Chairman (Ciel herself) performs Stage 3 **inline** — no subagent. See `council/CHAIRMAN.md` and `prompts/council/chairman_synthesis.md`.

**Responsibilities:**
1. Convene members per scope.
2. Anonymize Stage 1 outputs for Stage 2.
3. Synthesize Stage 3 verdict with full rationale.
4. Commit the decision + artifact to git.
5. Log to `activity.log` and append `CHANGELOG.md` entry if the decision mutates `~/.ciel/`.
6. Handle deadlocks per `council/ESCALATION.md`.

**Meta-judgment (deadlock path only):**
- Cannot override a Safety/Inclusion veto.
- Cannot raise a sub-threshold weighted score above pass without new evidence.
- May lower a passing score into deadlock if Chairman identifies coherent grounds (rare; logged prominently).
- If Chairman still cannot decide → escalate to user.

**Pivotal lens:** The lens whose score deviation contributed most to the verdict (positive or negative). Recorded for long-term improvement signal tracking.

---

## 6. Veto Conditions

### Architecture Council — Safety Veto (Absolute)

A **Safety score ≤ 3** is a hard block regardless of other votes.

**Hard veto conditions** (see `rubrics/VETO_CONDITIONS.md` — locked):
1. Untrusted code execution (runs arbitrary code from internet without sandboxing)
2. Credential exfiltration (reads env vars, keychain, `.env` to external sinks)
3. Unbounded destructive ops (`rm -rf`, `git reset --hard origin`, `DROP TABLE` without user consent)
4. Permission bypass (calls a tool in the host's `deny` list via an aliased name)
5. License incompatibility (cannot combine with Apache-2.0)
6. No origin provenance (Tier 3 acquisition with no source URL and no sandbox trace)
7. Known-vulnerable dependency (CVE above threshold)
8. Irreversible operation without guard clause (publish to immutable registry, send email)
9. Council/Constitution attack (modifies `council/` or `core/CONSTITUTION.md` outside amendment procedure)
10. PII handling without policy

**Evidence requirement:** A Safety veto must cite at least one concrete piece of evidence (file snippet, dep list entry, origin URL mismatch). "Gut feel" vetoes are rejected by Chairman as malformed.

**Override procedure:** Requires full Constitutional amendment (Council re-vote + explicit user confirmation). Never bypassed via escalation.

### Design Council — Inclusion Veto (Absolute)

An **Inclusion score ≤ 3** is a hard block regardless of other votes. It signals that the design excludes a meaningful group of users or creates safety/privacy harm for vulnerable users.

**Override:** Requires a conscious, user-confirmed decision to deprioritize accessibility.

### Accuracy-as-Ceiling Rule

Safety's score acts as a ceiling on the Chairman's verdict authority:
- Safety ≤ 3 → reject, no override
- Safety 4–5 → weighted pass only if average threshold met
- Safety ≥ 6 → normal synthesis applies

---

## 7. Recording Dockets

Every Council run produces a **docket** — a permanent record of the deliberation.

**Docket file:** `~/.ciel/council/dockets/DOCKET_<YYYYMMDD>_<SUBJECT>.md`

**Docket structure:**
```markdown
# COUNCIL DOCKET: <YYYYMMDD>_<SUBJECT>

**Date**: <date>
**Candidate Artifact**: <artifact_path>
**Scope**: `invocation_scopes/<SCOPE>.md`

---

## Stage 1: Independent Member Evaluation
### 1. <Member> (`members/<MEMBER>.md`)
- **Score**: X/10
- **Rationale**: ...
- **Flags**: [...]

[... all 5 members ...]

---

## Stage 2: Cross-Review & Anonymized Delta Check
- **Member 1 (<Lens>)**: ... (Final: X)
[... all 5 members ...]

---

## Stage 3: Chairman Synthesis & Voting Result
- **Voting Tally**: X/5 Pass votes
- **Weighted Score**: X.XX / 10.0
- **Safety/Inclusion Veto Check**: PASS|FAIL
- **Decision**: PASSED & RATIFIED | REJECTED | DEADLOCKED

---
**Status**: APPROVED BY COUNCIL OF FIVE | REJECTED | ESCALATED
```

**Docket index:** `~/.ciel/council/DOCKET_INDEX.json` tracks all dockets with id, subject, date, and status.

**Post-docket actions:**
- On pass: commit artifact to git, update registry, log to `activity.log`, append `CHANGELOG.md` if `~/.ciel/` mutated.
- On reject: archive candidate, document rejection reason, log.
- On deadlock: escalate per `council/ESCALATION.md`.

**No repeated escalation without new evidence:** If the identical artifact recurs, Ciel surfaces the previous decision and does not re-run Council unless explicitly asked.

---

## 8. Cost

~10 model calls per Council run (5 Stage 1 + 5 Stage 2). Ciel batches runs where possible and amortizes via prompt caching of personas and rubrics.

---

## 9. Quorum Rules

- Minimum 3 non-abstaining members for any decision.
- Safety (Architecture) / Inclusion (Design) must be present and non-abstaining.
- If the veto member is absent/timed-out, the run is re-attempted once; second failure escalates.
- Scores without rationale are discarded (treated as abstention).

---

## 10. Escalation

When the Council cannot reach a clean verdict (deadlock), escalation follows `council/ESCALATION.md`:

| Case | Handling |
| --- | --- |
| 2-2-1 (abstention) | Chairman re-dispatches the abstaining member once. If still abstaining → meta-judgment. |
| Scores clustered around threshold ± 1.0 | Chairman meta-judgment |
| Meta-judgment cannot decide | Escalate to user |
| Safety veto + Evolution strong-pass | Auto-reject; log contradiction as self-improvement signal |
| Stage 1 timeout ≥ 3 members | Rerun Stage 1 once; if still failing → escalate |

**Escalation never overrides a Safety/Inclusion veto.** It only resolves deadlocks, deadlock-adjacent scoring, or abstention deadlocks.

**User decisions** are recorded in `~/.ciel/council/<run_id>/user_decision.json` and count as signals for improvement (pattern: when does user override Council?).
