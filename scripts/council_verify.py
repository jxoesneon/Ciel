#!/usr/bin/env python3
"""Council run verifier — member-as-subagent enforcement (M1).

Validates a council run's artifact completeness under
``~/.ciel/council/<run_id>/`` and emits the run's structured member
verdicts to ``~/.ciel/improvements/signals/council-<run_id>.json`` so the
improvement loop can consume them.

Contract (see adapters/devin/COUNCIL_INVOCATION.md):
    spawn_receipts.json   {"mode": "subagent"|"inline", "members": {...}}
    members/<member>.stage1.json   {"member","stage":1,"score",...}
    members/<member>.stage2.json   {"member","stage":2,"score",...}
    verdict.json                   chairman docket with "votes" + "verdict"

``mode: "inline"`` is a declared degraded fallback — the run verifies but
is flagged ``unverified_member_isolation``. A run with no receipts at all
fails verification: member isolation is unproven.

Usage: council_verify.py <run_id|run_dir>
Exit: 0 verified (warnings allowed), 1 failed.
"""

import json
import sys
from pathlib import Path

MEMBERS = ["coherence", "capability", "safety", "efficiency", "evolution"]
CIEL = Path.home() / ".ciel"


def _load_json(path: Path) -> dict | None:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def verify(run_dir: Path) -> dict:  # noqa: PLR0912
    problems: list[str] = []
    warnings: list[str] = []

    receipts = _load_json(run_dir / "spawn_receipts.json")
    if receipts is None:
        problems.append("missing spawn_receipts.json — member isolation unproven")
        mode = "unknown"
    else:
        mode = receipts.get("mode", "unknown")
        if mode == "inline":
            warnings.append("unverified_member_isolation: run was inline, not subagent-dispatched")
        elif mode != "subagent":
            warnings.append(f"unrecognized spawn mode '{mode}'")

    member_verdicts = {}
    for member in MEMBERS:
        for stage in (1, 2):
            path = run_dir / "members" / f"{member}.stage{stage}.json"
            data = _load_json(path)
            if data is None:
                problems.append(f"missing/invalid {path.name}")
                continue
            if data.get("member") != member:
                problems.append(f"{path.name}: member field is '{data.get('member')}'")
            if data.get("stage") != stage:
                problems.append(f"{path.name}: stage field is {data.get('stage')}")
            score = data.get("score")
            if not isinstance(score, (int, float)):
                problems.append(f"{path.name}: non-numeric score {score!r}")
            else:
                member_verdicts.setdefault(member, {})[f"stage{stage}"] = score

    verdict = _load_json(run_dir / "verdict.json")
    if verdict is None:
        problems.append("missing/invalid verdict.json")
    else:
        votes = verdict.get("votes") or {}
        missing_votes = [m for m in MEMBERS if m not in votes]
        if missing_votes:
            problems.append(f"verdict.json missing votes for: {', '.join(missing_votes)}")
        safety = votes.get("safety")
        if verdict.get("verdict") == "pass" and isinstance(safety, (int, float)) and safety <= 3:
            problems.append("verdict=pass but safety <= 3 (veto condition)")

    return {
        "run_dir": str(run_dir),
        "mode": mode,
        "verified": not problems,
        "problems": problems,
        "warnings": warnings,
        "member_verdicts": member_verdicts,
    }


def emit_signal(run_id: str, result: dict) -> Path | None:
    signals = CIEL / "improvements" / "signals"
    try:
        signals.mkdir(parents=True, exist_ok=True)
        slug = run_id if run_id.startswith("council-") else f"council-{run_id}"
        out = signals / f"{slug}.json"
        out.write_text(json.dumps({
            "signal": "council_verdict",
            "run_id": run_id,
            "mode": result["mode"],
            "verified": result["verified"],
            "member_verdicts": result["member_verdicts"],
            "warnings": result["warnings"],
        }, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        return out
    except OSError:
        return None


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__.strip().splitlines()[0], file=sys.stderr)
        return 2
    arg = Path(sys.argv[1])
    run_dir = arg if arg.is_dir() else CIEL / "council" / sys.argv[1]
    if not run_dir.is_dir():
        print(f"[council_verify] no such run: {run_dir}")
        return 1

    result = verify(run_dir)
    run_id = run_dir.name
    signal_path = emit_signal(run_id, result)

    status = "VERIFIED" if result["verified"] else "FAILED"
    print(f"[council_verify] {run_id}: {status} (mode={result['mode']})")
    for w in result["warnings"]:
        print(f"  warning: {w}")
    for p in result["problems"]:
        print(f"  problem: {p}")
    if signal_path:
        print(f"  signal: {signal_path}")
    return 0 if result["verified"] else 1


if __name__ == "__main__":
    sys.exit(main())
