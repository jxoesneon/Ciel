#!/usr/bin/env python3
"""Review surface for the System-1 shadow log.

Reads ``~/.ciel/system1/events.jsonl`` and prints records the banding layer
marked ``flag`` or ``uncertain`` — the advisory tier's review queue.
Never a deny path; this is where shadow evidence becomes visible.

Usage:
  system1_review.py                 # flagged + uncertain records
  system1_review.py --all           # every record
  system1_review.py --surface NAME  # one surface only
  system1_review.py --stats         # band counts per surface
"""

import argparse
import json
import os
import sys
from pathlib import Path


def _home() -> Path:
    return Path(os.environ.get("CIEL_HOME") or Path.home() / ".ciel")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--surface")
    ap.add_argument("--stats", action="store_true")
    ap.add_argument("--log", type=Path,
                    default=_home() / "system1" / "events.jsonl")
    args = ap.parse_args()

    if not args.log.is_file():
        print("[system1_review] no events.jsonl — no shadow traffic yet")
        return 0

    shown = 0
    stats: dict[str, dict[str, int]] = {}
    for line in args.log.read_text(encoding="utf-8").splitlines():
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            continue
        surface = rec.get("surface") or "unknown"
        band = rec.get("flag") or "pass"
        stats.setdefault(surface, {}).setdefault(band, 0)
        stats[surface][band] += 1
        if args.surface and surface != args.surface:
            continue
        if not args.all and band == "pass":
            continue
        answers = ((rec.get("system1") or {}).get("answers") or {})
        detail = {
            k: {"choice": v.get("choice"), "confidence": v.get("confidence")}
            for k, v in answers.items() if isinstance(v, dict)
        }
        print(json.dumps({
            "ts": rec.get("ts"), "surface": surface, "flag": band,
            "meta": rec.get("meta"), "answers": detail,
            "cache_hit": rec.get("cache_hit"),
        }, ensure_ascii=False))
        shown += 1

    if args.stats or shown == 0:
        for surface, bands in sorted(stats.items()):
            parts = " ".join(f"{b}={n}" for b, n in sorted(bands.items()))
            print(f"[stats] {surface}: {parts}")
    if not args.stats:
        print(f"[system1_review] {shown} record(s) shown")
    return 0


if __name__ == "__main__":
    sys.exit(main())
