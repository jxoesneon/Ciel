#!/usr/bin/env python3
"""Durable-artifact attribution scan — fires on publish-ish commands.

Triggered by the ``publish_artifact_scan`` advisory rule in policy.yaml
(``scan: attribution``). Scans the command string plus the staged diff /
unpushed commit messages for AI-attribution tells the no-attribution
mandate forbids in durable artifacts.

Gate mode (default ``shadow``):
  * ``shadow``  — findings are logged to activity.log and surfaced as a
    warning; the command proceeds.
  * ``enforce`` — findings block the command.

Resolution order: ``CIEL_ATTRIBUTION_GATE`` env, then
``~/.ciel/risk/attribution_gate`` (one word: shadow|enforce).

Bypass: ``CIEL_ATTRIBUTION_SKIP=1`` present in the command environment
string — logged as ``bypass`` so use stays auditable.

Allowlist: ``~/.ciel/risk/attribution_allowlist.txt`` — one Python regex
per line; a flagged line matching any pattern is exempt (``#`` comments).
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

CIEL_HOME = Path(os.environ.get("CIEL_HOME") or (Path.home() / ".ciel"))

PATTERNS = {
    "attribution_trailer": re.compile(
        r"generated\s+with|co-authored-by\s*:", re.IGNORECASE
    ),
    "internal_identity": re.compile(
        r"«(?:Answer|Report|Notice|Council[^»]*)»|council\s+of\s+five", re.IGNORECASE
    ),
    "ai_emoji": re.compile(
        "[\U0001F300-\U0001FAFF\u2600-\u26FF\u2700-\u27BF]"
    ),
    "host_runtime": re.compile(r"\bdevin\b", re.IGNORECASE),
}

BYPASS_ENV = "CIEL_ATTRIBUTION_SKIP"
DEFAULT_MODE = "shadow"


def _gate_mode() -> str:
    mode = os.environ.get("CIEL_ATTRIBUTION_GATE", "").strip().lower()
    if mode in ("shadow", "enforce"):
        return mode
    try:
        mode = (CIEL_HOME / "risk" / "attribution_gate").read_text().strip().lower()
    except OSError:
        mode = ""
    return mode if mode in ("shadow", "enforce") else DEFAULT_MODE


def _allowlist() -> list[re.Pattern]:
    patterns = []
    try:
        for line in (CIEL_HOME / "risk" / "attribution_allowlist.txt").read_text(
            encoding="utf-8"
        ).splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            try:
                patterns.append(re.compile(line, re.IGNORECASE))
            except re.error:
                continue
    except OSError:
        pass
    return patterns


def _exempt(line: str, allowlist: list[re.Pattern]) -> bool:
    return any(rx.search(line) for rx in allowlist)


def _git(args: list[str]) -> str:
    try:
        out = subprocess.run(
            ["git", *args],
            capture_output=True, text=True, timeout=10,
        )
        return out.stdout if out.returncode == 0 else ""
    except (OSError, subprocess.TimeoutExpired):
        return ""


def collect_text(command: str) -> dict[str, list[str]]:
    """Return {source: [lines]} to scan for the given publish command."""
    sources: dict[str, list[str]] = {"command": command.splitlines() or [command]}

    if re.search(r"\bgit\s+commit\b", command):
        diff = _git(["diff", "--cached", "--unified=0"])
        added = [
            l[1:] for l in diff.splitlines()
            if l.startswith("+") and not l.startswith("+++")
        ]
        if added:
            sources["staged_diff"] = added

    if re.search(r"\bgit\s+(push|tag)\b", command):
        log = _git(["log", "--format=%B%x00", "@{u}..HEAD"])
        if log:
            sources["unpushed_messages"] = [
                l for l in log.split("\x00") if l.strip()
            ]

    return sources


def scan(command: str) -> dict:
    if BYPASS_ENV in command or os.environ.get(BYPASS_ENV):
        return {"result": "bypass", "findings": [], "mode": _gate_mode()}

    allowlist = _allowlist()
    findings = []
    for source, lines in collect_text(command).items():
        for line in lines:
            for category, rx in PATTERNS.items():
                if rx.search(line) and not _exempt(line, allowlist):
                    findings.append({
                        "category": category,
                        "source": source,
                        "line": line.strip()[:120],
                    })
                    break
    return {
        "result": "flagged" if findings else "clean",
        "findings": findings[:25],
        "mode": _gate_mode(),
    }


def main() -> int:
    command = sys.stdin.read()
    print(json.dumps(scan(command), ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
