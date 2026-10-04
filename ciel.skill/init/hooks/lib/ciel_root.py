#!/usr/bin/env python3
"""Shared Ciel install-root, binary, and activity-log primitives.

Single source for the discovery orders used by both the production hooks
(``init/hooks/*/pre_tool_use.sh``) and the agent-facing skill scripts
(``skills/ciel/scripts/*.py``). Resolves against the real installed layout
(``~/.ciel``) as well as the source repo layout (``ciel.skill/``).

Resolution orders:
  hook lib:  $CIEL_HOOK_LIB -> $CIEL_ROOT/hooks/lib ->
             $CIEL_ROOT/ciel.skill/init/hooks/lib -> $CIEL_HOME|~/.ciel/hooks/lib
             -> ~/.ciel/ciel.skill/init/hooks/lib -> repo ciel.skill/init/hooks/lib
  ciel bin:  $CIEL_BIN -> ~/.ciel/bin/ciel -> ~/.cargo/bin/ciel ->
             <init>/bin/ciel (repo/bundle) -> PATH
  roots:     $CIEL_ROOT -> $CIEL_HOME|~/.ciel -> repo root (ciel.skill ancestor)
"""

import json
import os
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

_LIB_DIR = Path(__file__).resolve().parent
_INIT_DIR = _LIB_DIR.parents[1] if _LIB_DIR.parent.name == "hooks" else None


def hook_lib_dirs() -> list[Path]:
    """Candidate directories that may contain hooks/lib (risk_policy etc.)."""
    out = []
    env_lib = os.environ.get("CIEL_HOOK_LIB")
    if env_lib:
        out.append(Path(env_lib))
    env_root = os.environ.get("CIEL_ROOT")
    if env_root:
        root = Path(env_root)
        out += [root / "hooks" / "lib",
                root / "ciel.skill" / "init" / "hooks" / "lib"]
    home = Path(os.environ.get("CIEL_HOME", str(Path.home() / ".ciel")))
    out += [home / "hooks" / "lib",
            home / "ciel.skill" / "init" / "hooks" / "lib"]
    # Repo/bundle layout: this file lives at <root>/ciel.skill/init/hooks/lib.
    if _INIT_DIR is not None:
        out.append(_LIB_DIR)
    return out


def find_hook_lib() -> Path | None:
    """First candidate dir that actually contains risk_policy.py."""
    for d in hook_lib_dirs():
        if (d / "risk_policy.py").is_file():
            return d
    return None


def candidate_roots() -> list[Path]:
    """Ordered Ciel install roots for locating shared assets (scripts/, risk/).

    Repo roots point at the directory containing ``ciel.skill`` so callers can
    reach either ``<root>/scripts`` (repo) or ``<root>/hooks`` (installed).
    """
    out = []
    env_root = os.environ.get("CIEL_ROOT")
    if env_root:
        out.append(Path(env_root))
    out.append(Path(os.environ.get("CIEL_HOME", str(Path.home() / ".ciel"))))
    for ancestor in _LIB_DIR.parents:
        if (ancestor / "ciel.skill").is_dir():
            out.append(ancestor)
            break
    return out


def ciel_home() -> Path:
    override = os.environ.get("CIEL_HOME")
    return Path(override) if override else Path.home() / ".ciel"


def ciel_bin() -> str | None:
    """Locate the ``ciel`` binary using the same order as the production
    hooks (init/hooks/*/pre_tool_use.sh), with PATH as a final resort.

    A non-empty ``CIEL_BIN`` is authoritative — exactly as in the hook
    bodies, where a set-but-unexecutable override skips the binary leg
    rather than silently rescuing a different (possibly stale) binary."""
    env_bin = os.environ.get("CIEL_BIN")
    if env_bin:
        return env_bin if os.access(env_bin, os.X_OK) else None
    home = ciel_home()
    candidates = [home / "bin" / "ciel", Path.home() / ".cargo" / "bin" / "ciel"]
    if _INIT_DIR is not None:
        candidates.append(_INIT_DIR / "bin" / "ciel")
    for c in candidates:
        if c.is_file() and os.access(c, os.X_OK):
            return str(c)
    return shutil.which("ciel")


def append_log(entry: dict) -> bool:
    """Append one JSON line to <ciel_home>/activity.log — the byte-compatible
    writer shared by every hook (json.dumps ensure_ascii=False + newline).
    Runs the deployed rotation policy afterwards. Never raises."""
    try:
        home = ciel_home()
        home.mkdir(parents=True, exist_ok=True)
        with (home / "activity.log").open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        return False
    try:
        import activity_log_rotate
        activity_log_rotate.rotate(ciel_home(), datetime.now(timezone.utc))
    except Exception:
        pass
    return True


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def bootstrap_lib() -> Path | None:
    """Insert the resolved hooks/lib into sys.path; returns the dir or None."""
    lib = find_hook_lib()
    if lib is not None and str(lib) not in sys.path:
        sys.path.insert(0, str(lib))
    return lib
