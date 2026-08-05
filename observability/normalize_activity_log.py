#!/usr/bin/env python3
"""Normalize ~/.ciel/activity.log to the spec JSONL format defined in
observability/ACTIVITY_LOG.md.

Reads the current ~/.ciel/activity.log, converts every parseable line to the
spec schema, skips non-JSON multi-line fragments, and writes the result to
~/.ciel/activity.log.normalized. Prints statistics at the end.
"""

from __future__ import annotations

import json
import os
import re
import sys
from collections import Counter
from datetime import datetime, timezone

HOME = os.path.expanduser("~")
SRC = os.path.join(HOME, ".ciel", "activity.log")
DST = os.path.join(HOME, ".ciel", "activity.log.normalized")

# ---------------------------------------------------------------------------
# Spec-schema defaults
# ---------------------------------------------------------------------------
DEFAULT_TOKENS = {"in": 0, "out": 0, "cached": 0}


def base_entry(ts: str, kind: str, op: str, **overrides) -> dict:
    """Build a spec-schema entry with sensible defaults."""
    entry = {
        "ts": ts,
        "session": "pre-migration",
        "runtime": "devin-for-terminal",
        "project": None,
        "kind": kind,
        "op": op,
        "risk": "low",
        "path": "n/a",
        "skill": None,
        "decision": "n/a",
        "duration_ms": 0,
        "tokens": {"in": 0, "out": 0, "cached": 0},
        "cost_usd": 0.0,
        "commit": None,
        "council_run": None,
        "notes": "",
        "redacted": [],
    }
    entry.update(overrides)
    return entry


# ---------------------------------------------------------------------------
# Hook-schema -> spec mapping
# ---------------------------------------------------------------------------
HOOK_TO_KIND = {
    "PreToolUse": "permission",
    "PostToolUse": "permission",
    "SessionStart": "health",
    "PermissionRequest": "permission",
    "bootstrap": "health",
}

# decision normalization
DECISION_MAP = {
    "approve": "allow",
    "allow": "allow",
    "block": "deny",
    "deny": "deny",
    "ask": "ask",
    "defer": "defer",
    "proceed": "proceed",
    "reject": "reject",
    "revise": "revise",
    "abort": "abort",
    "pass": "pass",
}


def norm_decision(d):
    if d is None:
        return "n/a"
    return DECISION_MAP.get(str(d).lower(), str(d).lower())


def hook_schema_to_spec(obj: dict) -> dict:
    ts = obj.get("ts", datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"))
    hook = obj.get("hook", "unknown")
    kind = HOOK_TO_KIND.get(hook, "permission")
    tool = obj.get("tool") or obj.get("op") or "unknown"
    op = str(tool)
    decision = norm_decision(obj.get("decision"))
    reason = obj.get("reason") or ""
    command = obj.get("command") or ""
    notes_parts = []
    if reason:
        notes_parts.append(str(reason))
    if command:
        notes_parts.append("cmd: " + str(command)[:200])
    notes = " | ".join(notes_parts)[:240]
    return base_entry(
        ts,
        kind,
        op,
        decision=decision,
        notes=notes,
    )


# ---------------------------------------------------------------------------
# Pipe-separated -> spec mapping
# ---------------------------------------------------------------------------
PIPE_KIND_MAP = {
    "ROUTER_INIT": "health",
    "ROUTER_FAST_PATH": "route",
    "ROUTER_REASONING_PATH": "route",
    "ROUTER_ACQUISITION_PATH": "route",
    "ROUTER_USER_ESCALATION": "escalation",
    "COUNCIL_INIT": "council",
    "COUNCIL_GATE": "council",
    "RISK_INIT": "health",
    "CONFIG_DOMAIN_INIT": "health",
    "MEMORY_OBSERVABILITY_INIT": "health",
    "RUNTIME_DETECTION": "health",
    "REGISTRY_INIT": "health",
    "ACQUISITION_INIT": "acquisition",
    "SELF_IMPROVEMENT_COMPLETE": "improvement",
    "INTEGRITY_CHECK": "health",
    "SEED_SKILLS_VERIFIED": "health",
    "TEMPLATES_PROMPTS_VERIFIED": "health",
    "CIEL_IMPLEMENTATION_COMPLETE": "health",
}

PIPE_LINE_RE = re.compile(r"^(\d{4}-\d{2}-\d{2}T[0-9:Z+\-.]+)\|([A-Z_]+)\|?(.*)$")


def pipe_to_spec(line: str) -> dict | None:
    m = PIPE_LINE_RE.match(line)
    if not m:
        return None
    ts, kind_raw, rest = m.group(1), m.group(2), m.group(3)
    # rest may contain detail|extra
    parts = rest.split("|", 1) if rest else []
    detail = parts[0] if parts else ""
    extra = parts[1] if len(parts) > 1 else ""
    kind = PIPE_KIND_MAP.get(kind_raw, "health")
    op = kind_raw.lower()
    notes = (detail + (" | " + extra if extra else "")).strip()[:240]
    return base_entry(ts, kind, op, notes=notes)


# ---------------------------------------------------------------------------
# Spec-schema detection
# ---------------------------------------------------------------------------
SPEC_REQUIRED = {"ts", "kind", "op"}


def is_spec_schema(obj: dict) -> bool:
    return SPEC_REQUIRED.issubset(obj.keys()) and "hook" not in obj


def normalize_spec(obj: dict) -> dict:
    """Ensure a spec-schema entry has all required fields with defaults."""
    entry = base_entry(
        obj.get("ts", datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")),
        obj.get("kind", "health"),
        obj.get("op", "unknown"),
    )
    # overlay all provided fields
    for k, v in obj.items():
        if k in entry:
            entry[k] = v
    # guarantee tokens shape
    if not isinstance(entry.get("tokens"), dict):
        entry["tokens"] = DEFAULT_TOKENS
    else:
        t = entry["tokens"]
        entry["tokens"] = {
            "in": int(t.get("in", 0) or 0),
            "out": int(t.get("out", 0) or 0),
            "cached": int(t.get("cached", 0) or 0),
        }
    return entry


# ---------------------------------------------------------------------------
# Other JSON -> best-effort spec
# ---------------------------------------------------------------------------
def other_json_to_spec(obj: dict) -> dict:
    ts = obj.get("ts", datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"))
    # if it has a 'kind' use it, else infer
    kind = obj.get("kind")
    if kind and kind in {
        "route", "acquisition", "council", "mutation", "escalation",
        "health", "merge", "permission", "error", "improvement",
        "sweep", "backup", "trace",
    }:
        pass
    elif "hook" in obj:
        kind = HOOK_TO_KIND.get(obj["hook"], "permission")
    else:
        kind = "health"
    op = obj.get("op") or obj.get("tool") or obj.get("event") or "unknown"
    notes = ""
    for nk in ("reason", "command", "detail", "message", "event", "version"):
        if nk in obj and obj[nk]:
            notes = str(obj[nk])[:240]
            break
    decision = norm_decision(obj.get("decision"))
    return base_entry(ts, kind, str(op), decision=decision, notes=notes)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> int:
    if not os.path.exists(SRC):
        print(f"ERROR: source log not found: {SRC}", file=sys.stderr)
        return 1

    total_in = 0
    normalized = 0
    skipped = 0
    by_kind: Counter = Counter()
    by_source: Counter = Counter()

    with open(SRC, "r", encoding="utf-8", errors="replace") as fin, \
         open(DST, "w", encoding="utf-8") as fout:
        for line in fin:
            total_in += 1
            raw = line.rstrip("\n")
            stripped = raw.strip()
            if not stripped:
                skipped += 1
                continue

            # Try pipe-separated first (starts with ISO timestamp + |)
            if PIPE_LINE_RE.match(stripped):
                entry = pipe_to_spec(stripped)
                if entry is not None:
                    fout.write(json.dumps(entry, separators=(",", ":")) + "\n")
                    normalized += 1
                    by_kind[entry["kind"]] += 1
                    by_source["pipe"] += 1
                    continue
                # fall through if pipe parse failed

            # Try JSON
            if stripped.startswith("{"):
                try:
                    obj = json.loads(stripped)
                except json.JSONDecodeError:
                    # broken JSON / multi-line fragment
                    skipped += 1
                    continue
                if not isinstance(obj, dict):
                    skipped += 1
                    continue
                if is_spec_schema(obj):
                    entry = normalize_spec(obj)
                    by_source["spec"] += 1
                elif "hook" in obj:
                    entry = hook_schema_to_spec(obj)
                    by_source["hook"] += 1
                else:
                    entry = other_json_to_spec(obj)
                    by_source["other_json"] += 1
                fout.write(json.dumps(entry, separators=(",", ":")) + "\n")
                normalized += 1
                by_kind[entry["kind"]] += 1
                continue

            # Non-JSON, non-pipe -> skip (multi-line fragment)
            skipped += 1
            by_source["skipped_nonjson"] += 1

    print("=" * 60)
    print("Activity Log Normalization Report")
    print("=" * 60)
    print(f"Source:      {SRC}")
    print(f"Destination: {DST}")
    print(f"Total in:    {total_in}")
    print(f"Normalized:  {normalized}")
    print(f"Skipped:     {skipped}")
    print()
    print("By source format:")
    for src_name, cnt in by_source.most_common():
        print(f"  {src_name:20s} {cnt}")
    print()
    print("By kind:")
    for k, cnt in by_kind.most_common():
        print(f"  {k:20s} {cnt}")
    print("=" * 60)
    return 0


if __name__ == "__main__":
    sys.exit(main())
