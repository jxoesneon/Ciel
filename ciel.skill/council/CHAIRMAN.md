# CHAIRMAN

Ciel presides. The Chairman synthesizes final verdicts.

## Responsibilities

1. Convene members per scope.
2. Anonymize Stage 1 outputs for Stage 2 (`ANONYMIZATION.md`).
3. Synthesize Stage 3 verdict with full rationale.
4. Commit the decision + artifact to git.
5. Log to `activity.log` and append `CHANGELOG.md` entry if the decision mutates `~/.ciel/`.
6. Handle deadlocks per `ESCALATION.md`.

## Synthesis Algorithm

Input: five final scores + rationales. Output: `{ verdict: pass|reject|deadlock, weighted_score, pivotal_lens, summary }`.

```text
weighted_score =
  0.20 * coherence

+ 0.20 * capability
+ 0.25 * safety          # heavier weight
+ 0.15 * efficiency
+ 0.20 * evolution

if safety <= 3: verdict = reject; return
if weighted_score >= pass_threshold AND at least 3 of 5 score >= pass_score: verdict = pass
elif weighted_score < reject_threshold: verdict = reject
else: verdict = deadlock
```

Thresholds in `rubrics/SCORING.md`.

## Pivotal Lens

The lens whose score deviation contributed most to the verdict (positive or negative). Recorded for long-term improvement signal tracking (`self_improvement/TRIGGERS.md`).

## Chairman's Meta-Judgment

In the deadlock path, Chairman may apply meta-judgment using `prompts/council/chairman_synthesis.md`. This is bounded:

- cannot override a Safety veto,
- cannot raise a sub-threshold weighted score above pass without new evidence,
- may lower a passing score into deadlock if the Chairman identifies coherent grounds (rare; logged prominently).

If Chairman still cannot decide, escalate.

## Output Shape

```json
{
  "verdict": "pass",
  "weighted_score": 7.6,
  "pivotal_lens": "capability",
  "votes": { "coherence": 7, "capability": 9, "safety": 7, "efficiency": 6, "evolution": 8 },
  "chairman_summary": "...",
  "commit_sha": "abc123..."
}
```

Serialized to `~/.ciel/council/<run_id>.json` and mirrored into MemPalace partition `ciel/council/`.
