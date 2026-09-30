#!/usr/bin/env python3
"""Export System-1 shadow traffic as RLCD training pairs.

Reads ``~/.ciel/system1/events.jsonl`` and emits JSONL records
``{state, question_key, chosen, rejected, surface, weak, source, margin}`` suitable
for fine-tuning a domain checkpoint (RLCD / contrastive preference).

Key Features:
  - Streamed line-by-line reading for O(1) constant memory.
  - Deterministic secret scrubbing of all state fields to prevent credential leakage.
  - Multi-surface outcome harvesting across pre_tool_risk, router, completion_check, and council_prescreen.
  - Calibrated filtering by --min-confidence, --min-margin, and --surface.

Usage:
  system1_export.py [--log FILE] [--out FILE] [--surface NAME]
                    [--min-confidence FLOAT] [--min-margin FLOAT]
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "ciel.skill" / "init" / "hooks" / "lib"
if str(LIB) not in sys.path:
    sys.path.insert(0, str(LIB))

try:
    from secret_scan import PATTERNS
except ImportError:
    PATTERNS = {
        "github_token": r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b|\bgithub_pat_[A-Za-z0-9_]{20,}\b",
        "aws_access_key": r"\bAKIA[0-9A-Z]{16}\b",
        "api_key_prefixed": r"\b(?:sk|pk|key|api|tok)_[A-Za-z0-9_-]{20,}\b|\bsk-[A-Za-z0-9_-]{20,}\b",
        "private_key_block": r"-----BEGIN [A-Z ]*PRIVATE KEY",
        "slack_token": r"\bxox[baprs]-[0-9A-Za-z-]{10,}\b",
        "gcp_api_key": r"\bAIza[0-9A-Za-z_-]{35}\b",
        "jwt": r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b",
        "npm_token": r"\bnpm_[A-Za-z0-9]{36}\b",
        "crates_token": r"\bcio[0-9A-Za-z]{25,}\b",
        "password_assignment": r"(?i)\b(?:sudo\s+)?(?:password|passwd|passphrase)\s*(?:is|:|=)\s*['\"]?[^\s'\"]{4,}",
        "secret_assignment": r"(?i)\b(?:api[_-]?key|access[_-]?token|auth[_-]?token|secret[_-]?key|client[_-]?secret)\s*[:=]\s*['\"]?[A-Za-z0-9_\-]{8,}",
        "generic_secret_kv": r"(?i)\b(?:token|secret)\s+is\s+['\"]?[A-Za-z0-9_\-]{8,}",
    }

_COMPILED_SECRETS = {name: re.compile(pat) for name, pat in PATTERNS.items()}

RISK_LABEL = {
    "deny": "dangerous",
    "allow_overridden": "dangerous",
    "allow": "safe",
}


def _home() -> Path:
    return Path(os.environ.get("CIEL_HOME") or Path.home() / ".ciel")


def scrub_secrets(val: any) -> any:
    """Recursively redact secrets from state fields."""
    if isinstance(val, str):
        s = val
        for cat, rx in _COMPILED_SECRETS.items():
            s = rx.sub(f"[REDACTED_SECRET:{cat.upper()}]", s)
        return s
    elif isinstance(val, dict):
        return {k: scrub_secrets(v) for k, v in val.items()}
    elif isinstance(val, list):
        return [scrub_secrets(x) for x in val]
    return val


def _extract_truth(surface: str, meta: dict) -> tuple[str | None, bool, str]:
    """Returns (truth_label, is_weak, source_str)."""
    if "expected" in meta and meta["expected"] is not None:
        return str(meta["expected"]), False, "meta.expected"

    if surface == "pre_tool_risk":
        reg = meta.get("regex_decision")
        if reg:
            truth = RISK_LABEL.get(reg)
            return truth, (reg == "allow"), "regex_decision"
    elif surface in ("router", "router_selection"):
        skill = meta.get("chosen_skill") or meta.get("successful_skill")
        if skill:
            return str(skill), False, "router_outcome"
    elif surface == "completion_check":
        if "verified" in meta:
            truth = "complete" if meta["verified"] else "incomplete"
            return truth, False, "completion_outcome"
    elif surface == "council_prescreen":
        cons = meta.get("council_consensus")
        if cons:
            truth = "routine" if cons == "approve" else "escalate"
            return truth, False, "council_consensus"

    return None, False, "unknown"


def _compute_margin(probs: dict) -> float:
    if not probs or len(probs) < 2:
        return 0.0
    sorted_probs = sorted(probs.values(), reverse=True)
    return float(sorted_probs[0] - sorted_probs[1])


def _pairs(rec: dict) -> list[dict]:
    surface = rec.get("surface") or "unknown"
    meta = rec.get("meta") or {}
    answers = ((rec.get("system1") or {}).get("answers") or {})
    raw_state = rec.get("state") or {}
    scrubbed_state = scrub_secrets(raw_state)

    truth, weak, source = _extract_truth(surface, meta)
    if not truth:
        return []

    out = []
    for qkey, answer in answers.items():
        if not isinstance(answer, dict):
            continue
        probs = answer.get("probabilities") or {}
        if truth not in (probs or {truth: 1}):
            continue

        rejected = [o for o in probs if o != truth] or [
            o for o in ("safe", "dangerous", "routine", "escalate", "complete", "incomplete")
            if o != truth
        ]
        margin = _compute_margin(probs)

        out.append({
            "surface": surface,
            "question_key": qkey,
            "state": scrubbed_state,
            "state_ref": meta.get("ts"),
            "chosen": truth,
            "rejected": rejected,
            "model_choice": answer.get("choice"),
            "model_confidence": answer.get("confidence"),
            "margin": margin,
            "weak": weak,
            "source": source,
        })
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--log", type=Path,
                    default=_home() / "system1" / "events.jsonl")
    ap.add_argument("--out", type=Path,
                    default=_home() / "system1" / "rlcd_pairs.jsonl")
    ap.add_argument("--surface", type=str, default="",
                    help="filter by surface (e.g. pre_tool_risk, router, completion_check)")
    ap.add_argument("--min-confidence", type=float, default=0.0,
                    help="drop pairs where model confidence is below this")
    ap.add_argument("--min-margin", type=float, default=0.0,
                    help="drop pairs where top-1 probability margin is below this")
    args = ap.parse_args()

    if not args.log.is_file():
        print("[system1_export] no events.jsonl")
        return 1

    args.out.parent.mkdir(parents=True, exist_ok=True)
    written = skipped = 0

    # Stream line-by-line for constant O(1) memory consumption
    with args.log.open("r", encoding="utf-8") as in_fh, args.out.open("w", encoding="utf-8") as out_fh:
        for line in in_fh:
            if not line.strip():
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError:
                continue

            if args.surface and rec.get("surface") != args.surface:
                continue

            for pair in _pairs(rec):
                conf = pair.get("model_confidence")
                if isinstance(conf, (int, float)) and conf < args.min_confidence:
                    skipped += 1
                    continue
                margin = pair.get("margin", 0.0)
                if isinstance(margin, (int, float)) and margin < args.min_margin:
                    skipped += 1
                    continue

                out_fh.write(json.dumps(pair, ensure_ascii=False) + "\n")
                written += 1

    print(f"[system1_export] wrote {written} pairs "
          f"(skipped {skipped}) -> {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
