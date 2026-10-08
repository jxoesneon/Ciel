#!/usr/bin/env python3
"""Static security scan for skills, wrapping `uvx skillfrisk`.

Aggregates per-skill `skillfrisk scan <dir> --json` results into one report.
Advisory by default: if the scanner is unavailable the gate passes with a
warning (use --strict to fail instead). Exit 2 when any skill reports
`failed: true` (skillfrisk's own threshold).

Usage:
    scan_skills.py [paths...] [--all] [--fail-on high|any|none]
                  [--report PATH] [--baseline PATH] [--no-baseline] [--strict]

A baseline file (default: ciel.skill/risk/skill_scan_baseline.json when it
exists) lists individually reviewed findings — each entry needs a
justification and matches on skill + rule_id (+ optional path). Baselined
findings are reported under "accepted" and no longer count toward failure.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

SEVERITY_ORDER = {"none": 0, "any": 1, "high": 2}


def _scan_cmd(dir_path: Path, report: Path | None = None) -> list[str]:
    override = os.environ.get("SCAN_SKILLS_CMD")
    if override:
        return override.split() + [str(dir_path), "--json"]
    return ["uvx", "skillfrisk", "scan", str(dir_path), "--json"]


def _tool_available() -> bool:
    return bool(os.environ.get("SCAN_SKILLS_CMD")) or bool(shutil.which("uvx"))


def _scan_one(dir_path: Path) -> dict:
    try:
        proc = subprocess.run(
            _scan_cmd(dir_path), capture_output=True, text=True, check=False
        )
        if proc.returncode != 0 and not proc.stdout.strip():
            return {"skill": dir_path.name, "error": proc.stderr.strip() or f"exit {proc.returncode}"}
        data = json.loads(proc.stdout)
        data["skill"] = dir_path.name
        return data
    except (OSError, json.JSONDecodeError) as exc:
        return {"skill": dir_path.name, "error": str(exc)}


def _skill_dirs(root: Path) -> list[Path]:
    dirs = []
    for base in (root / "skills", root / "ciel.skill" / "seed_skills"):
        if base.is_dir():
            dirs.extend(sorted(p for p in base.iterdir() if (p / "SKILL.md").is_file()))
    return dirs


def _load_baseline(path: Path) -> list[dict]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"[scan] baseline unreadable ({exc}); ignoring", file=sys.stderr)
        return []
    entries = data.get("accepted", [])
    return [e for e in entries if isinstance(e, dict) and e.get("skill") and e.get("rule_id")]


def _baselined(skill: str, finding: dict, baseline: list[dict]) -> dict | None:
    for entry in baseline:
        if entry["skill"] != skill or entry["rule_id"] != finding.get("rule_id"):
            continue
        if entry.get("path") and entry["path"] != finding.get("path"):
            continue
        return entry
    return None


def _severity_rank(findings: list) -> int:
    rank = 0
    for f in findings:
        sev = (f.get("severity") or "").lower() if isinstance(f, dict) else ""
        rank = max(rank, SEVERITY_ORDER.get(sev, 1))
    return rank


def main() -> int:  # noqa: PLR0912, PLR0915
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, help="skill dirs to scan")
    parser.add_argument("--all", action="store_true",
                        help="scan every skill under skills/ and ciel.skill/seed_skills/")
    parser.add_argument("--fail-on", choices=["high", "any", "none"], default="none",
                        help="additionally fail on findings at this severity")
    parser.add_argument("--report", type=Path, help="write aggregated JSON report to PATH")
    parser.add_argument("--baseline", type=Path,
                        help="accepted-findings JSON (default: ciel.skill/risk/skill_scan_baseline.json)")
    parser.add_argument("--no-baseline", action="store_true",
                        help="ignore the baseline file even if present")
    parser.add_argument("--strict", action="store_true",
                        help="exit 2 when the scanner is unavailable instead of warning")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    baseline_path = args.baseline
    if baseline_path is None and not args.no_baseline:
        default = root / "ciel.skill" / "risk" / "skill_scan_baseline.json"
        if default.is_file():
            baseline_path = default
    baseline = _load_baseline(baseline_path) if baseline_path else []
    targets = list(args.paths)
    if args.all:
        targets.extend(_skill_dirs(root))
    if not targets:
        print("[scan] no skill dirs given (use --all or pass paths)", file=sys.stderr)
        return 0 if not args.strict else 2

    if not _tool_available():
        msg = "[scan] skillfrisk unavailable (uvx missing); advisory pass"
        if args.strict:
            print(msg + " — strict mode: failing", file=sys.stderr)
            return 2
        print(msg)
        return 0

    scanned = 0
    raw_failed = set()
    findings_by_skill = {}
    accepted = []
    errors = []
    for d in targets:
        result = _scan_one(d)
        if "error" in result:
            errors.append(result)
            print(f"[scan] {d.name}: scanner error: {result['error']}", file=sys.stderr)
            continue
        scanned += 1
        if result.get("failed"):
            raw_failed.add(d.name)
        remaining = []
        for finding in result.get("findings") or []:
            entry = _baselined(d.name, finding, baseline)
            if entry is not None:
                accepted.append({"skill": d.name, "finding": finding,
                                 "justification": entry.get("justification", "")})
            else:
                remaining.append(finding)
        if remaining:
            findings_by_skill[d.name] = remaining

    # A skill fails when skillfrisk flagged it or its findings meet the
    # --fail-on severity threshold — counting only non-baselined findings.
    threshold = SEVERITY_ORDER[args.fail_on]
    failed = sorted(
        name for name, findings in findings_by_skill.items()
        if name in raw_failed or (threshold > 0 and _severity_rank(findings) >= threshold)
    )

    report = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "scanned": scanned,
        "failed": failed,
        "findings_by_skill": findings_by_skill,
        "accepted": accepted,
        "capability": {
            "tool": "skillfrisk",
            "invocation": "uvx skillfrisk scan <dir> --json",
            "fail_on": args.fail_on,
            "baseline": str(baseline_path) if baseline_path else None,
            "errors": errors,
        },
    }
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    total_findings = sum(len(v) for v in findings_by_skill.values())
    print(
        f"[scan] scanned={scanned} failed={len(failed)} findings={total_findings}"
        + (f" accepted={len(accepted)}" if accepted else "")
        + (f" errors={len(errors)}" if errors else "")
    )
    return 2 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
