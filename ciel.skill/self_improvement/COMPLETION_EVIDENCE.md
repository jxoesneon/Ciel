# COMPLETION_EVIDENCE — Task-Class → Artifact Matrix

Docket `council-20260923-conversation-audit`, mechanism M5. "Done" is not a
claim — it is a claim **plus the verification artifact its task class
requires**. The audit's most expensive recurring correction was premature
completion ("still only the first frame", "features were missed"), so this
matrix makes evidence decidable per class before any enforcement is
considered.

## Matrix

| Task class | Required evidence artifact | Rejected as evidence |
| --- | --- | --- |
| Code change / bugfix | Fresh full-suite test output post-change (not a prior run) | "tests should pass", partial suite |
| Feature / UX | Live probe of the running artifact — screenshot, frame update, API response, e2e trace | static code review, "implemented" status |
| Config / install / service | Verification command output on the target after restart/reload | install script exit 0 alone |
| Docs | Docs-vs-code diff check: claims in docs verified against implementation | spellcheck, linter on prose |
| Release / publish | Coverage report (~100% new/changed code) + Council sign-off on release diff | changelog exists |
| Policy / hook / gate change | Fired-and-logged live event through the deployed path + regression corpus cases | unit test of the rule alone |
| Migration / port | 1:1 parity check vs the reference implementation | "equivalent behavior" assertion |

## Contract

1. The requirement ledger (`hooks/lib/requirements.py`) holds session asks;
   the Stop hook reconciles it — pending items block a clean "done".
2. Each ledger item's resolution should name its evidence artifact; the
   task-class table defines what counts.
3. Stop-hook reminders are capped at **2 per session** — enforcement must
   never loop (council Safety condition).
4. `system1.py --decide` `completion_check` surface remains the advisory
   second opinion; the ledger is the deterministic record.
5. When evidence artifacts are emitted under
   `~/.ciel/improvements/signals/` or `~/.ciel/checkpoints/`, the
   paired-eval and outcome-scoring loops can consume them downstream.
