#!/usr/bin/env python3
"""Post-task completion and verification gate using System-1 Jev decision tier.

Evaluates objective deliverables vs. empirical evidence artifacts according to the
task-class matrix defined in COMPLETION_EVIDENCE.md:
  - code_change: fresh full-suite test output post-change (not prior run)
  - feature: live probe / screenshot / API response / e2e trace
  - config: verification command output on target after restart/reload
  - docs: docs-vs-code diff check
  - release: coverage report + council sign-off
  - policy: fired-and-logged live event through deployed path + regression cases
  - migration: 1:1 parity check vs reference implementation

Speaks Jev protocol POST /v1/systemone with typed choice ('complete'/'incomplete')
and 1-5 quality rubric score.

Fail-open: If System-1 is offline, unreachable, or disabled, returns exit 0 with
advisory notice, preserving developer and CI velocity.

Exit codes:
  0: verified complete, or fail-open when System-1 is offline/unreachable
  2: completion gate failed (unverified claims / false pass detected in enforce mode)
"""

import argparse
import json
import os
import sys
from pathlib import Path

# hooks/lib sits one level up in every layout: repo (init/scripts ->
# init/hooks/lib), bundle (init/scripts -> init/hooks/lib), and installed
# (~/.ciel/scripts -> ~/.ciel/hooks/lib).
LIB = Path(__file__).resolve().parent.parent / "hooks" / "lib"
sys.path.insert(0, str(LIB))

import system1

TASK_CLASSES = {
    "code_change": "Fresh full-suite test output post-change (not a prior run)",
    "feature": "Live probe of running artifact — screenshot, frame update, API response, e2e trace",
    "config": "Verification command output on target after restart/reload",
    "docs": "Docs-vs-code diff check: claims in docs verified against implementation",
    "release": "Coverage report (~100% new/changed code) + Council sign-off on release diff",
    "policy": "Fired-and-logged live event through deployed path + regression corpus cases",
    "migration": "1:1 parity check vs reference implementation",
}


def verify_completion(objective: str, evidence: str, task_class: str = "code_change",
                      gate: str = "enforce", timeout: float = 5.0) -> dict:
    """Run System-1 completion check on objective vs empirical evidence."""
    res = system1.completion_check(
        objective=objective,
        evidence=evidence,
        task_class=task_class,
        with_score=True,
        timeout=timeout,
    )

    if res is None:
        # Fail-open semantics
        return {
            "status": "fail_open_pass",
            "degraded": True,
            "decision": "allow",
            "verified": True,
            "choice": None,
            "score": None,
            "band": "pass",
            "confidence": None,
            "reason": "System-1 decision tier is offline or disabled; failing open.",
        }

    choice = res.get("choice")
    band = res.get("band", "pass")
    score = res.get("score")
    confidence = res.get("confidence", 0.0)

    is_false_pass = (choice == "incomplete" or band == "flag" or (score is not None and score < 4))

    if is_false_pass:
        decision = "deny" if gate == "enforce" else "allow"
        verified = False
        reason = f"System-1 flagged completion as incomplete (confidence {confidence:.2f}, score {score}/5)"
    else:
        decision = "allow"
        verified = True
        reason = f"System-1 verified completion (confidence {confidence:.2f}, score {score}/5)"

    return {
        "status": "evaluated",
        "degraded": False,
        "decision": decision,
        "verified": verified,
        "choice": choice,
        "score": score,
        "band": band,
        "confidence": confidence,
        "model": res.get("model"),
        "reason": reason,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--objective", type=str, help="Task objective or acceptance criteria")
    parser.add_argument("--objective-file", type=Path, help="Path to file containing objective")
    parser.add_argument("--evidence", type=str, help="Empirical verification evidence or command output")
    parser.add_argument("--evidence-file", type=Path, help="Path to file containing verification evidence")
    parser.add_argument("--task-class", choices=list(TASK_CLASSES.keys()), default="code_change",
                        help="Task classification from COMPLETION_EVIDENCE.md")
    parser.add_argument("--gate", choices=["shadow", "enforce"],
                        default=os.environ.get("CIEL_COMPLETION_GATE", "enforce"),
                        help="Gate enforcement mode (shadow logs only; enforce blocks unverified completion)")
    parser.add_argument("--json", action="store_true", help="Output result as JSON")
    parser.add_argument("--timeout", type=float, default=5.0, help="System-1 request timeout in seconds")
    args = parser.parse_args()

    objective = args.objective or ""
    if args.objective_file and args.objective_file.is_file():
        objective = args.objective_file.read_text(encoding="utf-8")

    evidence = args.evidence or ""
    if args.evidence_file and args.evidence_file.is_file():
        evidence = args.evidence_file.read_text(encoding="utf-8")

    if not objective:
        # Try reading from stdin if piped
        if not sys.stdin.isatty():
            evidence = sys.stdin.read()
        if not objective:
            objective = "Unspecified task objective"

    result = verify_completion(
        objective=objective,
        evidence=evidence,
        task_class=args.task_class,
        gate=args.gate,
        timeout=args.timeout,
    )

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        status_sym = "✓" if result["verified"] else "✗"
        print(f"[{status_sym}] Completion Verification ({args.task_class}): {result['reason']}")
        if result["choice"]:
            print(f"    Choice: {result['choice']} | Band: {result['band']} | Score: {result['score']}/5 | Gate: {args.gate}")

    if result["decision"] == "deny":
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
