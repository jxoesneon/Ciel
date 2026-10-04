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
4. System-1 `completion_check` (`system1.py`, `system1.rs::evaluate_completion`,
   `ciel.skill/init/scripts/verify_completion.py`) evaluates objective deliverables vs. empirical
   evidence using typed choice (`complete` / `incomplete`) and typed 1–5 rubric
   scores. While the ledger remains the deterministic record, System-1 actively
   validates that claimed resolutions match actual artifacts.
5. In `scripts/paired_eval.py` (`--completion-gate [shadow|enforce]`) and post-task
   verification scripts (`skills/ciel/scripts/verify_evidence.py`), System-1 is
   actively invoked to detect false passes (e.g. exit code 0 without required
   test execution or live probes) or unverified assertions.
6. **Fail-Open Semantics**: If System-1 is offline, unreachable, disabled
   (`CIEL_SYSTEM1_DISABLED=1`), or times out, all completion gates fail open,
   retaining the deterministic test runner result and preventing blocked workflows.
