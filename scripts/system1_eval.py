#!/usr/bin/env python3
"""Calibration harness for the System-1 decision tier.

Scores the configured endpoint (``CIEL_SYSTEM1_URL``, default the local
laya-serve) against per-surface labeled corpora and writes
``risk/system1_calibration.json`` — the committed evidence artifact that any
surface promotion must cite.

Corpora live in ``tests/fixtures/system1_<surface>_cases.json`` (the
pre_tool_risk surface reuses ``hook_redteam_cases.json``). Each corpus may
carry a ``candidates`` map used to build the router's choice criteria.

Binary surfaces report confusion/precision/recall/F1 on the positive class,
confidence means, latency, and per-tau operating-point sweeps. The router
surface reports top-1 accuracy and mean confidence margin instead.

Usage:
  python3 scripts/system1_eval.py                 # all surfaces, write artifact
  python3 scripts/system1_eval.py --surface NAME  # one surface
  python3 scripts/system1_eval.py --stdout        # print, don't write
"""

import argparse
import json
import os
import re
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "ciel.skill" / "init" / "hooks" / "lib"))

import risk_policy
import system1

OUT = ROOT / "ciel.skill" / "risk" / "system1_calibration.json"
FIXTURES = ROOT / "tests" / "fixtures"

EXPECT_LABEL = {"allow": "safe", "deny": "dangerous",
                "allow_overridden": "dangerous"}

PRESCREEN_QUESTIONS = system1.PRESCREEN_QUESTIONS


RAW_STATE = False  # --raw-state: A/B the plain {tool,command,path} state


def _risk_state(case: dict) -> dict:
    if RAW_STATE:
        return {"tool": case.get("tool") or "",
                "command": case.get("command") or "",
                "path": case.get("path") or ""}
    return system1.tool_state(case.get("tool") or "",
                              case.get("command") or "",
                              case.get("path") or "")


def _prescreen_state(case: dict) -> dict:
    return {"event": case.get("event") or ""}


def _router_state(case: dict) -> dict:
    return {"task": case.get("task") or ""}


def _router_questions(corpus: dict) -> dict:
    return {"route": {"type": "choice",
                      "instructions": "Which skill should handle this task?",
                      "criteria": corpus.get("candidates") or {}}}


def _skill_dirs() -> list:
    dirs = [Path(os.environ.get("CIEL_SKILLS_DIR") or "")
            if os.environ.get("CIEL_SKILLS_DIR") else None,
            Path.home() / ".ciel" / "skills",
            ROOT / "skills"]
    return [d for d in dirs if d and d.is_dir()]


def _registry_candidates() -> dict:
    """{skill_id: description} from the first populated skills dir — the
    real ~200-skill registry rather than the corpus's hand-picked nine."""
    for d in _skill_dirs():
        out = {}
        for skill_md in sorted(d.glob("*/SKILL.md")):
            desc = ""
            try:
                head = skill_md.read_text(encoding="utf-8",
                                          errors="replace")[:2000]
            except OSError:
                continue
            m = re.search(r"(?m)^description:\s*(.+)$", head)
            if m:
                desc = m.group(1).strip()
            else:
                for line in head.splitlines():
                    line = line.strip()
                    if line and not line.startswith(("#", "---", "name:")):
                        desc = line
                        break
            out[skill_md.parent.name] = desc
        if out:
            return out
    return {}


_REGISTRY_CACHE: dict = {}


def _router_registry_questions(case: dict, corpus: dict) -> dict:
    if not _REGISTRY_CACHE:
        _REGISTRY_CACHE.update(_registry_candidates())
    criteria = system1.shortlist_options(case.get("task") or "",
                                         _REGISTRY_CACHE, k=10)
    return {"route": {"type": "choice",
                      "instructions": "Which skill should handle this task?",
                      "criteria": criteria}}


# surface -> {corpus, questions(static dict or fn(corpus)), state, truth,
#             positive label for binary metrics | None for multiclass}
SURFACES = {
    "pre_tool_risk": {
        "corpus": FIXTURES / "hook_redteam_cases.json",
        "questions": lambda c: risk_policy.SYSTEM1_QUESTIONS,
        "state": _risk_state,
        "truth": lambda c: EXPECT_LABEL.get(c.get("expect")),
        "positive": "dangerous",
    },
    "council_prescreen": {
        "corpus": FIXTURES / "system1_prescreen_cases.json",
        "questions": lambda c: PRESCREEN_QUESTIONS,
        "state": _prescreen_state,
        "truth": lambda c: c.get("expected"),
        "positive": "escalate",
    },
    "router": {
        "corpus": FIXTURES / "system1_router_cases.json",
        "questions": _router_questions,
        "state": _router_state,
        "truth": lambda c: c.get("expected"),
        "positive": None,  # multiclass: accuracy + margin
    },
    # Real-registry routing: same corpus tasks, but candidates come from the
    # full skills dir and are reduced per-case via lexical shortlist.
    "router_registry": {
        "corpus": FIXTURES / "system1_router_cases.json",
        "questions_for_case": _router_registry_questions,
        "state": _router_state,
        "truth": lambda c: c.get("expected"),
        "positive": None,
    },
}


def _predict(answers: dict) -> str | None:
    """Single-question verdict: the choice of the first answer. For the
    two-question risk variant, 'dangerous' from any question wins."""
    choices = [a.get("choice") for a in answers.values()
               if isinstance(a, dict)]
    if "dangerous" in choices:
        return "dangerous"
    return choices[0] if choices else None


def _confidence(answers: dict) -> float:
    confs = [a.get("confidence") for a in answers.values()
             if isinstance(a, dict) and isinstance(a.get("confidence"), (int, float))]
    return max(confs) if confs else 0.0


def evaluate_surface(name: str, spec: dict, timeout: float) -> dict:
    corpus = json.loads(spec["corpus"].read_text(encoding="utf-8"))
    cases = corpus["cases"] if isinstance(corpus, dict) else corpus
    questions_for_case = spec.get("questions_for_case")
    questions = (questions_for_case is None and spec["questions"](corpus))

    tp = fp = tn = fn = errors = correct = 0
    in_options_total = in_options_hits = 0
    conf_correct, conf_wrong, margins, latencies = [], [], [], []
    per_case = []
    for case in cases:
        truth = spec["truth"](case)
        if truth is None:
            continue
        if questions_for_case is not None:
            questions = questions_for_case(case, corpus)
        t0 = time.monotonic()
        result = system1.ask(spec["state"](case), questions, timeout=timeout)
        latencies.append(time.monotonic() - t0)
        if result is None:
            errors += 1
            per_case.append({"id": case.get("id"), "truth": truth,
                             "pred": None})
            continue
        answers = result["answers"]
        pred = _predict(answers)
        conf = _confidence(answers)
        probs = {}
        for a in answers.values():
            if isinstance(a, dict) and isinstance(a.get("probabilities"), dict):
                probs.update(a["probabilities"])
        margin = None
        if pred in probs:
            others = [v for k, v in probs.items() if k != pred]
            margin = probs[pred] - (max(others) if others else 0.0)
            margins.append(margin)
        per_case.append({"id": case.get("id"), "truth": truth, "pred": pred,
                         "confidence": conf, "margin": margin})
        pos = spec["positive"]
        if pos is None:
            # did the truth survive candidate reduction (shortlist recall)?
            offered = set()
            for q in questions.values():
                if isinstance(q, dict) and isinstance(q.get("criteria"), dict):
                    offered.update(q["criteria"])
            if offered:
                in_options_total += 1
                in_options_hits += int(truth in offered)
            correct += int(pred == truth)
            (conf_correct if pred == truth else conf_wrong).append(conf)
            continue
        if pred == pos and truth == pos:
            tp += 1
        elif pred == pos:
            fp += 1
        elif truth == pos:
            fn += 1
        else:
            tn += 1
        (conf_correct if pred == truth else conf_wrong).append(conf)

    base = {
        "surface": name,
        "cases": len(per_case),
        "errors": errors,
        "mean_confidence_correct": (sum(conf_correct) / len(conf_correct)
                                    if conf_correct else None),
        "mean_confidence_wrong": (sum(conf_wrong) / len(conf_wrong)
                                  if conf_wrong else None),
        "mean_latency_s": sum(latencies) / len(latencies),
        "mean_margin": (sum(margins) / len(margins) if margins else None),
        "per_case": per_case,
    }
    if spec["positive"] is None:
        base["accuracy"] = (correct / len(per_case)) if per_case else None
        if in_options_total:
            base["shortlist_recall"] = in_options_hits / in_options_total
        return base

    precision = tp / (tp + fp) if tp + fp else None
    recall = tp / (tp + fn) if tp + fn else None
    base.update({
        "confusion": {"tp": tp, "fp": fp, "tn": tn, "fn": fn},
        "precision_" + spec["positive"]: precision,
        "recall_" + spec["positive"]: recall,
        "f1_" + spec["positive"]: (
            2 * precision * recall / (precision + recall)
            if precision and recall else None),
        "sweep_flag_uncertain_negative": _uncertain_sweep(
            per_case, spec["positive"]),
    })
    return base


def _uncertain_sweep(per_case: list[dict], positive: str) -> list[dict]:
    """Flag low-confidence negative-class predictions as uncertain for
    review: how many misses (truth=positive, pred=negative) does tau catch,
    and how many benign negatives get flagged."""
    sweep = []
    for tau in (0.01, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4):
        caught = flagged = missed = 0
        for c in per_case:
            conf = c.get("confidence") or 0.0
            if c["pred"] is None or c["pred"] == positive:
                continue
            if c["truth"] == positive:
                if conf < tau:
                    caught += 1
                else:
                    missed += 1
            elif conf < tau:
                flagged += 1
        sweep.append({"tau": tau, "missed_caught": caught,
                      "missed_remaining": missed,
                      "negative_flagged": flagged})
    return sweep


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--surface", choices=sorted(SURFACES))
    ap.add_argument("--timeout", type=float, default=30.0)
    ap.add_argument("--stdout", action="store_true")
    ap.add_argument("--raw-state", action="store_true",
                    help="A/B: send plain {tool,command,path} instead of "
                         "the enriched tool_state()")
    args = ap.parse_args()
    global RAW_STATE
    RAW_STATE = args.raw_state

    names = [args.surface] if args.surface else list(SURFACES)
    report = {
        "generated": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "endpoint": system1._url(),
        "model": system1._model() or "(endpoint default)",
        "surfaces": {},
    }
    for name in names:
        spec = SURFACES[name]
        if not spec["corpus"].is_file():
            report["surfaces"][name] = {"error": f"missing {spec['corpus']}"}
            continue
        report["surfaces"][name] = evaluate_surface(name, spec, args.timeout)
    rendered = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
    if args.stdout:
        print(rendered, end="")
    else:
        OUT.write_text(rendered, encoding="utf-8")
        for name, s in report["surfaces"].items():
            if "error" in s:
                print(f"[system1_eval] {name}: {s['error']}")
                continue
            metric = (f"accuracy={s['accuracy']:.2f}" if "accuracy" in s else
                      f"P={s.get('precision_dangerous') or s.get('precision_escalate')} "
                      f"R={s.get('recall_dangerous') or s.get('recall_escalate')}")
            print(f"[system1_eval] {name}: cases={s['cases']} "
                  f"errors={s['errors']} {metric}")
        print(f"[system1_eval] -> {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
