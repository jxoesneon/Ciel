#!/usr/bin/env python3
"""
Evolution & Cognitive Architect: RLCD Pipeline
Generates preference pairs from System-1 flags, Council signals, and Dockets.
"""

import json
import os
import sys
from pathlib import Path

# Import secret scrubbing from existing export script
sys.path.insert(0, str(Path(__file__).resolve().parent))
try:
    from system1_export import scrub_secrets
except ImportError:
    def scrub_secrets(val): return val

def _home() -> Path:
    return Path(os.environ.get("CIEL_HOME") or Path.home() / ".ciel")

def process_events(events_file: Path) -> list:  # noqa: PLR0912
    pairs = []
    if not events_file.is_file():
        return pairs

    with open(events_file, encoding="utf-8") as f:
        for line in f:
            if not line.strip():
                continue
            try:
                rec = json.loads(line)
            except ValueError:
                continue

            surface = rec.get("surface")
            meta = rec.get("meta", {})
            sys1 = rec.get("system1", {}) or {}
            answers = sys1.get("answers", {}) if isinstance(sys1, dict) else {}
            flag = rec.get("flag", "pass")
            state = scrub_secrets(rec.get("state", {}))

            # 1. System-1 Flagged Command (Overriding explicit allow/pass)
            if flag in ("flag", "uncertain") and surface == "pre_tool_risk":
                for qkey, ans in answers.items():
                    if isinstance(ans, dict) and ans.get("choice") == "dangerous":
                        pairs.append({
                            "surface": surface,
                            "question_key": qkey,
                            "state": state,
                            "chosen": "dangerous",
                            "rejected": ["safe"],
                            "source": "system1_flag",
                            "model_confidence": ans.get("confidence")
                        })

            # 2. Context surfaces — a flagged/uncertain shadow verdict
            # departs from the passive default; prefer the verdict
            # direction, same convention as pre_tool_risk above.
            if flag in ("flag", "uncertain"):
                if surface == "context_select":
                    ans = answers.get("relevant") or {}
                    if ans.get("choice") == "drop":
                        pairs.append({
                            "surface": surface, "question_key": "relevant",
                            "state": state, "chosen": "drop",
                            "rejected": ["keep"], "source": "system1_flag",
                            "model_confidence": ans.get("confidence")})
                elif surface == "memory_salience":
                    ans = answers.get("salience") or {}
                    if ans.get("choice") == "skip":
                        pairs.append({
                            "surface": surface, "question_key": "salience",
                            "state": state, "chosen": "skip",
                            "rejected": ["store"], "source": "system1_flag",
                            "model_confidence": ans.get("confidence")})
                elif surface == "context_compaction":
                    ans = answers.get("action") or {}
                    ch = ans.get("choice")
                    if ch and ch != "continue":
                        pairs.append({
                            "surface": surface, "question_key": "action",
                            "state": state, "chosen": ch,
                            "rejected": ["continue"], "source": "system1_flag",
                            "model_confidence": ans.get("confidence")})
                elif surface == "mandate_canary":
                    ans = answers.get("mandates") or {}
                    if ans.get("choice") == "drifted":
                        pairs.append({
                            "surface": surface, "question_key": "mandates",
                            "state": state, "chosen": "drifted",
                            "rejected": ["operative"], "source": "system1_flag",
                            "model_confidence": ans.get("confidence")})

            # 3. Council Override / Prescreen
            if surface == "council_prescreen":
                cons = meta.get("council_consensus")
                if cons in ("reject", "escalate"):
                    pairs.append({
                        "surface": surface,
                        "question_key": "council_review",
                        "state": state,
                        "chosen": "escalate",
                        "rejected": ["routine"],
                        "source": "council_override"
                    })
                elif cons == "approve":
                    pairs.append({
                        "surface": surface,
                        "question_key": "council_review",
                        "state": state,
                        "chosen": "routine",
                        "rejected": ["escalate"],
                        "source": "council_approval"
                    })
    return pairs

def process_signals_and_dockets(dockets_dir: Path, signals_dir: Path) -> list:  # noqa: PLR0912
    pairs = []

    # Audit Signals
    if signals_dir.is_dir():
        for sig_file in signals_dir.glob("*.json"):
            try:
                with open(sig_file, encoding="utf-8") as f:
                    data = json.load(f)
            except (OSError, ValueError):
                continue

            if data.get("signal") == "council_verdict":
                # Determine verdict from scores if present
                scores = data.get("member_verdicts", {})
                passed = True

                for stages in scores.values():
                    if isinstance(stages, dict) and stages.get("stage2", 10) <= 3:
                        passed = False

                # Check overall weighting (simple average approximation)
                if passed and scores:
                    avg = sum(s.get("stage2", 0) for s in scores.values() if isinstance(s, dict)) / len(scores)
                    if avg < 6.5:
                        passed = False

                chosen = "routine" if passed else "escalate"
                rejected = "escalate" if passed else "routine"

                pairs.append({
                    "surface": "council_audit",
                    "question_key": "audit_verdict",
                    "state": {
                        "run_id": data.get("run_id"),
                        "mode": data.get("mode"),
                        "member_verdicts": data.get("member_verdicts")
                    },
                    "chosen": chosen,
                    "rejected": [rejected],
                    "source": "council_signal_json"
                })

    # Council verdict files (~/.ciel/council/*.verdict.json) — the
    # Chairman-serialized shape: {"verdict": "pass"|"reject"|...}.
    council_dir = dockets_dir.parent if dockets_dir.name == "dockets" \
        else dockets_dir
    if council_dir.is_dir():
        for vfile in council_dir.glob("*.verdict.json"):
            try:
                with open(vfile, encoding="utf-8") as f:
                    data = json.load(f)
            except (OSError, ValueError):
                continue
            verdict = data.get("verdict")
            if verdict not in ("pass", "reject", "deadlock"):
                continue
            passed = verdict == "pass"
            pairs.append({
                "surface": "council_audit",
                "question_key": "audit_verdict",
                "state": {
                    "run_id": data.get("run_id") or vfile.stem,
                    "weighted_score": data.get("weighted_score"),
                    "votes": data.get("votes"),
                },
                "chosen": "routine" if passed else "escalate",
                "rejected": ["escalate" if passed else "routine"],
                "source": "council_verdict_json",
            })

    # Dockets
    if dockets_dir.is_dir():
        for docket in dockets_dir.glob("*.md"):
            content = docket.read_text(encoding="utf-8")
            if "VERDICT: REJECT" in content or "Veto Condition Triggered" in content:
                pairs.append({
                    "surface": "council_audit",
                    "question_key": "audit_verdict",
                    "state": {"docket": docket.name, "summary": "Council Vetoed or Rejected"},
                    "chosen": "escalate",
                    "rejected": ["routine"],
                    "source": "council_docket_md"
                })
            elif "VERDICT: APPROVE" in content or "Exemplary Pass" in content:
                pairs.append({
                    "surface": "council_audit",
                    "question_key": "audit_verdict",
                    "state": {"docket": docket.name, "summary": "Council Approved"},
                    "chosen": "routine",
                    "rejected": ["escalate"],
                    "source": "council_docket_md"
                })

    return pairs

def main():
    home = _home()
    events_file = home / "system1" / "events.jsonl"
    dockets_dir = home / "council" / "dockets"
    signals_dir = home / "improvements" / "signals"
    out_file = home / "system1" / "rlcd_pairs.jsonl"

    all_pairs = []
    all_pairs.extend(process_events(events_file))
    all_pairs.extend(process_signals_and_dockets(dockets_dir, signals_dir))

    # Write Out
    out_file.parent.mkdir(parents=True, exist_ok=True)
    with open(out_file, "w", encoding="utf-8") as f:
        f.writelines(json.dumps(p, ensure_ascii=False) + "\n" for p in all_pairs)

    print("[RLCD Pipeline] Processed events, dockets, and signals.")
    print(f"[RLCD Pipeline] Generated {len(all_pairs)} calibration pairs.")
    print(f"[RLCD Pipeline] Output saved to: {out_file}")

if __name__ == "__main__":
    main()
