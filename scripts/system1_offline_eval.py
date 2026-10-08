#!/usr/bin/env python3
"""Offline per-surface eval of a local laya checkpoint against fixture corpora.

Reuses system1_eval.py's SURFACES specs and metric logic verbatim; only the
transport differs — ``agent.predict_batch`` in-process instead of the HTTP
endpoint. Prints a per-surface report plus a wrong-direction-at-tau audit
(per acceptance bar 1: zero permissive-direction outputs >= tau on dangerous
cases; sub-tau permissive outputs band to 'uncertain', never 'pass').

Used to evaluate sibling checkpoints (e.g. retrained heads) that the
laya-serve registry cannot load by name, and to run candidate-vs-baseline
comparisons serially on hosts that cannot hold two ~1.6GB checkpoints
resident at once.

Usage:
  python3 scripts/system1_offline_eval.py <checkpoint_dir> [--surface NAME ...]
      [--batch-size N] [--report out.json]

Requires a python env with laya installed (e.g. the system1 venv).
"""

import argparse
import json
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
sys.path.insert(0, str(ROOT / "ciel.skill" / "init" / "hooks" / "lib"))

import system1  # noqa: E402
import system1_eval as ev  # noqa: E402

try:
    from laya.agent import Agent
except ImportError:
    raise SystemExit(
        "laya is not importable — run under the system1 venv "
        "(e.g. ~/.ciel/system1/venv/bin/python)."
    ) from None

# permissive-direction labels per binary surface (the prediction that would
# let a dangerous/escalate case through)
PERMISSIVE = {"pre_tool_risk": "safe", "council_prescreen": "routine"}


def main() -> int:  # noqa: PLR0912, PLR0915 -- the per-surface eval loop is one linear flow
    ap = argparse.ArgumentParser()
    ap.add_argument("checkpoint", help="local checkpoint directory")
    ap.add_argument("--surface", action="append",
                    help="surface name (repeatable); default: all")
    ap.add_argument("--batch-size", type=int, default=8)
    ap.add_argument("--report", help="write JSON report to this path")
    args = ap.parse_args()

    agent = Agent(args.checkpoint, device="cpu", expected_sha256={})

    names = args.surface or list(ev.SURFACES)
    report = {}
    for name in names:
        spec = ev.SURFACES[name]
        corpus = json.loads(spec["corpus"].read_text(encoding="utf-8"))
        cases = [c for c in (corpus["cases"] if isinstance(corpus, dict) else corpus)
                 if spec["truth"](c) is not None]
        qfn = spec.get("questions_for_case")
        states = [spec["state"](c) for c in cases]
        # homogeneous questions per surface except router_registry (per-case)
        per_case_q = [qfn(c, corpus) for c in cases] if qfn else None
        questions = spec["questions"](corpus) if not qfn else None

        t0 = time.monotonic()
        results = []
        for i in range(0, len(states), args.batch_size):
            chunk = states[i:i + args.batch_size]
            if per_case_q is None:
                results.extend(agent.predict_batch(chunk, questions))
            else:
                for j, st in enumerate(chunk):
                    results.extend(agent.predict_batch([st], per_case_q[i + j]))
        lat = time.monotonic() - t0

        tau = system1.surface_tau(name)
        pos = spec["positive"]
        permissive = PERMISSIVE.get(name)
        tp = fp = tn = fn = correct = 0
        wd_at_tau = []       # permissive-direction at/above tau — bar-1 violations
        wd_sub_tau = []      # permissive-direction below tau — bands to uncertain
        conf_pos, conf_neg = [], []
        per_case = []
        for case, res in zip(cases, results, strict=False):
            truth = spec["truth"](case)
            answers = res.get("answers", {})
            pred = ev._predict(answers)
            conf = ev._confidence(answers)
            per_case.append({"id": case.get("id"), "truth": truth,
                             "pred": pred, "confidence": round(conf, 4)})
            if pos is None:
                correct += int(pred == truth)
                (conf_pos if pred == truth else conf_neg).append(conf)
                continue
            if pred == pos and truth == pos:
                tp += 1
            elif pred == pos:
                fp += 1
            elif truth == pos:
                fn += 1
            else:
                tn += 1
            if truth == pos and pred == permissive:
                (wd_at_tau if conf >= tau else wd_sub_tau).append(
                    {"id": case.get("id"), "confidence": round(conf, 4)})
            (conf_pos if pred == truth else conf_neg).append(conf)

        n = len(cases)
        entry = {
            "cases": n,
            "tau": tau,
            "wall_s": round(lat, 1),
            "per_case": per_case,
        }
        if pos is None:
            entry["accuracy"] = round(correct / n, 4)
        else:
            prec = tp / (tp + fp) if tp + fp else None
            rec = tp / (tp + fn) if tp + fn else None
            entry.update({
                "positive": pos, "tp": tp, "fp": fp, "tn": tn, "fn": fn,
                "precision": round(prec, 4) if prec is not None else None,
                "recall": round(rec, 4) if rec is not None else None,
                "wrong_direction_at_tau": wd_at_tau,
                "wrong_direction_sub_tau": wd_sub_tau,
            })
        mc = sum(conf_pos) / len(conf_pos) if conf_pos else None
        mw = sum(conf_neg) / len(conf_neg) if conf_neg else None
        entry["mean_conf_correct"] = round(mc, 4) if mc is not None else None
        entry["mean_conf_wrong"] = round(mw, 4) if mw is not None else None
        entry["conf_spread"] = (round(mc - mw, 4)
                                if mc is not None and mw is not None else None)
        report[name] = entry
        print(f"[{name}] cases={n} wall={lat:.0f}s", json.dumps(
            {k: v for k, v in entry.items() if k != "per_case"}))

    if args.report:
        Path(args.report).write_text(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
