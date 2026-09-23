#!/usr/bin/env python3
"""Integrity sweep for a Ciel home (see init/INTEGRITY.md).

Compares the git-tracked file set against INTEGRITY.json and classifies each
path as ok / unknown-drift / expected-drift / missing / unexpected. Writes a
timestamped report under HOME/integrity/ and prints a one-line summary.

Usage: integrity.py [--home PATH] [--write] [--json]
"""

import argparse
import contextlib
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

MANIFEST = "INTEGRITY.json"
REPORT_DIR = "integrity"


def _git(home: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", "-C", str(home), *args],
        check=False,
        capture_output=True,
        text=True,
    )


def _tracked_files(home: Path) -> list[str]:
    proc = _git(home, "ls-files")
    if proc.returncode != 0:
        print(f"[integrity] {home} is not a git repository", file=sys.stderr)
        sys.exit(2)
    return [
        p for p in proc.stdout.splitlines()
        if p and p != MANIFEST and not p.startswith(REPORT_DIR + "/")
    ]


def _load_manifest(home: Path) -> dict[str, str]:
    """Return {path: sha256hex} from either legacy or spec manifest shape."""
    manifest = home / MANIFEST
    if not manifest.is_file():
        return {}
    try:
        data = json.loads(manifest.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    files = data.get("files", {})
    out = {}
    for path, entry in files.items():
        if isinstance(entry, str):
            out[path] = entry  # legacy flat-hex
        elif isinstance(entry, dict) and "sha256" in entry:
            out[path] = entry["sha256"]
    return out


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _git_dirty(home: Path, rel: str) -> bool:
    proc = _git(home, "status", "--porcelain", "--", rel)
    return bool(proc.stdout.strip())


def sweep(home: Path) -> dict:
    tracked = set(_tracked_files(home))
    manifest = _load_manifest(home)
    result = {
        "checked": 0,
        "ok": 0,
        "unknown_drift": [],
        "expected_drift": [],
        "missing": [],
        "unexpected": [],
    }
    for rel in sorted(tracked):
        result["checked"] += 1
        path = home / rel
        if not path.is_file():
            result["missing"].append(rel)
            continue
        digest = _sha256(path)
        recorded = manifest.get(rel)
        if recorded is None:
            result["unexpected"].append(rel)
        elif digest == recorded:
            result["ok"] += 1
        elif _git_dirty(home, rel):
            result["unknown_drift"].append(rel)
        else:
            result["expected_drift"].append(rel)
    return result


def _write_manifest(home: Path, tracked: list[str], now: str) -> None:
    version = "1.0.0"
    installed = home / "INSTALLED.json"
    if installed.is_file():
        with contextlib.suppress(OSError, ValueError):
            version = json.loads(installed.read_text(encoding="utf-8")).get("version", version)
    files = {}
    for rel in sorted(tracked):
        path = home / rel
        if path.is_file():
            files[rel] = {"sha256": _sha256(path), "size": path.stat().st_size}
    manifest = {
        "schema": 1,
        "version": version,
        "last_verified": now,
        "files": files,
    }
    (home / MANIFEST).write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    default_home = os.environ.get("CIEL_HOME", str(Path.home() / ".ciel"))
    parser.add_argument("--home", type=Path, default=Path(default_home))
    parser.add_argument("--write", action="store_true",
                        help="regenerate INTEGRITY.json in the spec shape")
    parser.add_argument("--json", action="store_true", help="print the full report JSON")
    args = parser.parse_args()
    home = args.home.resolve()

    now = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    now_iso = datetime.now(timezone.utc).isoformat()

    if args.write:
        tracked = _tracked_files(home)
        _write_manifest(home, tracked, now_iso)
        print(f"[integrity] wrote {MANIFEST} ({len(tracked)} tracked files)")
        return 0

    result = sweep(home)
    report = {"ts": now_iso, "sweep": result}
    report_dir = home / REPORT_DIR
    try:
        report_dir.mkdir(parents=True, exist_ok=True)
        (report_dir / f"{now}.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    except OSError:
        pass

    drift = len(result["unknown_drift"]) + len(result["expected_drift"])
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(
            f"[integrity] checked={result['checked']} ok={result['ok']} "
            f"drift={drift} missing={len(result['missing'])} "
            f"unexpected={len(result['unexpected'])}"
        )
    return 1 if result["unknown_drift"] or result["missing"] else 0


if __name__ == "__main__":
    sys.exit(main())
