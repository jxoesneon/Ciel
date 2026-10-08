#!/usr/bin/env python3
"""Jev-teacher labeling pass for the ciel-context corpus.

Reads the ambiguous teacher rows emitted by system1_corpus_build.py
(--teacher-out), asks hosted Jev for the verdict on each, and writes the
labeled rows to the output file. Fail-closed: any row Jev declines, errors
on, or answers below --min-confidence is written with label=null and never
silently labeled.

Usage:
  system1_teacher_label.py --in teacher.jsonl --out labeled.jsonl
      [--limit 150] [--min-confidence 0.4]

Hosted access goes through system1.py's own egress gates: _redact() on all
state payloads, _hosted_url/_hosted_key resolution, https enforcement.
Every call is one billable hosted ask — --limit caps total spend.
"""

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent /
                    "ciel.skill/init/hooks/lib"))
import system1  # noqa: E402

# QID → which label field Jev's answer populates.
_QID_OF = {
    "pre_tool_risk": "risk",
    "council_prescreen": "scope",
    "memory_salience": "salience",
    "context_compaction": "action",
    "mandate_canary": "mandates",
    "context_select": "relevant",
}


def teacher_ask(state: dict, questions: dict, timeout: float = 8.0):
    """One hosted-Jev ask through the gated, redacted path."""
    if not system1._hosted_active():
        return None, "hosted-inactive"
    endpoint = system1._endpoint_of(system1._hosted_url())
    result, status = system1._do_ask(
        endpoint, system1._hosted_key(), system1._hosted_model(),
        state, questions, timeout, redact=True)
    if result is None:
        return None, f"http-{status}" if status else "transport"
    return result, "ok"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="src", required=True)
    ap.add_argument("--out", dest="dst", required=True)
    ap.add_argument("--limit", type=int, default=150)
    ap.add_argument("--min-confidence", type=float, default=0.4)
    ap.add_argument("--dry-run", action="store_true",
                    help="validate rows and egress gates; zero hosted calls")
    args = ap.parse_args()

    rows = [json.loads(line) for line in open(args.src) if line.strip()]
    calls = 0
    n_ok = n_declined = 0
    with open(args.dst, "w") as out:
        for row in rows:
            surface = row["surface"]
            qid = _QID_OF[surface]
            if args.dry_run:
                row["teacher"] = "dry-run"
                row["label"] = None
                out.write(json.dumps(row, ensure_ascii=False) + "\n")
                continue
            if calls >= args.limit:
                row["teacher"] = "over-limit"
                row["label"] = None
                out.write(json.dumps(row, ensure_ascii=False) + "\n")
                continue
            calls += 1
            result, how = teacher_ask(row["state"], row["questions"])
            ans = ((result or {}).get("answers") or {}).get(qid) or {}
            choice = ans.get("choice")
            conf = float(ans.get("confidence") or 0.0)
            if choice and conf >= args.min_confidence:
                label = {qid: choice}
                # compaction rows also carry the score question
                if surface == "context_compaction":
                    score = ((result or {}).get("answers") or {}).get("pressure") or {}
                    if score.get("score") is not None:
                        label["pressure"] = score["score"]
                row["label"] = label
                row["teacher"] = f"jev:{conf:.3f}"
                n_ok += 1
            else:
                row["label"] = None
                row["teacher"] = f"declined:{how}:conf={conf:.3f}"
                n_declined += 1
            out.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(f"calls={calls} labeled={n_ok} declined={n_declined}",
          file=sys.stderr)


if __name__ == "__main__":
    main()
