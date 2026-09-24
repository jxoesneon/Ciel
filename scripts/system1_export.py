#!/usr/bin/env python3
"""Export System-1 shadow traffic as RLCD training pairs.

Reads ``~/.ciel/system1/events.jsonl`` and emits JSONL records
``{state, question_key, chosen, rejected, surface, weak, source}`` suitable
for fine-tuning a domain checkpoint (RLCD / contrastive preference).

Label sources, strongest first:
  - ``meta.expected`` — human/probe-provided ground truth
  - ``meta.regex_decision`` — deterministic policy verdict for
    pre_tool_risk (deny/allow_overridden -> 'dangerous'; 'allow' is a WEAK
    'safe' label: absence of a regex hit is not proof of safety)

Usage:
  system1_export.py [--out FILE] [--min-confidence FLOAT]
"""

import argparse
import json
import os
import sys
from pathlib import Path

RISK_LABEL = {"deny": "dangerous", "allow_overridden": "dangerous",
              "allow": "safe"}


def _home() -> Path:
    return Path(os.environ.get("CIEL_HOME") or Path.home() / ".ciel")


def _pairs(rec: dict) -> list[dict]:
    surface = rec.get("surface") or "unknown"
    meta = rec.get("meta") or {}
    answers = ((rec.get("system1") or {}).get("answers") or {})
    out = []
    for qkey, answer in answers.items():
        if not isinstance(answer, dict):
            continue
        probs = answer.get("probabilities") or {}
        truth = meta.get("expected")
        weak = False
        if truth is None and surface == "pre_tool_risk":
            truth = RISK_LABEL.get(meta.get("regex_decision") or "")
            weak = meta.get("regex_decision") == "allow"
        if truth is None or truth not in (probs or {truth: 1}):
            continue
        rejected = [o for o in probs if o != truth] or [
            o for o in ("safe", "dangerous", "routine", "escalate")
            if o != truth]
        out.append({
            "surface": surface,
            "question_key": qkey,
            "state": rec.get("state") or {},
            "state_ref": meta.get("ts"),
            "chosen": truth,
            "rejected": rejected,
            "model_choice": answer.get("choice"),
            "model_confidence": answer.get("confidence"),
            "weak": weak,
            "source": "meta.expected" if meta.get("expected") else
                      "regex_decision",
        })
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", type=Path,
                    default=_home() / "system1" / "events.jsonl")
    ap.add_argument("--out", type=Path,
                    default=_home() / "system1" / "rlcd_pairs.jsonl")
    ap.add_argument("--min-confidence", type=float, default=0.0,
                    help="drop pairs where model confidence is below this")
    args = ap.parse_args()

    if not args.log.is_file():
        print("[system1_export] no events.jsonl")
        return 1

    written = skipped = 0
    with args.out.open("w", encoding="utf-8") as fh:
        for line in args.log.read_text(encoding="utf-8").splitlines():
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue
            for pair in _pairs(rec):
                conf = pair.get("model_confidence")
                if (isinstance(conf, (int, float))
                        and conf < args.min_confidence):
                    skipped += 1
                    continue
                fh.write(json.dumps(pair, ensure_ascii=False) + "\n")
                written += 1
    print(f"[system1_export] wrote {written} pairs "
          f"(skipped {skipped}) -> {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
