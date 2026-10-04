#!/usr/bin/env python3
"""Locate the Ciel hooks/lib directory and put it on ``sys.path``.

Agent-facing scripts (``ciel_preflight.py``, ``ciel_audit.py``,
``verify_evidence.py``) import this first so the shared primitives in
``hooks/lib`` (``risk_policy``, ``ciel_root``, ``system1`` …) resolve
identically on every platform and in every layout:

  repo:      <repo>/skills/ciel/scripts -> <repo>/ciel.skill/init/hooks/lib
  installed: ~/.ciel/scripts/../hooks/lib, ~/.ciel/hooks/lib
  bundle:    init/scripts/../hooks/lib

The candidate order mirrors ``ciel_root.hook_lib_dirs`` — it is kept inline
here because that module lives in the very directory being searched for.
"""

import os
import sys
from pathlib import Path


def _candidates() -> list[Path]:
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
    here = Path(__file__).resolve().parent
    for anc in [here, *here.parents]:
        for rel in (Path("ciel.skill") / "init" / "hooks" / "lib",
                    Path("init") / "hooks" / "lib",
                    Path("hooks") / "lib"):
            cand = anc / rel
            if cand not in out:
                out.append(cand)
    return out


def load() -> Path | None:
    """Insert the first dir containing ``ciel_root.py`` into sys.path."""
    for d in _candidates():
        if (d / "ciel_root.py").is_file():
            s = str(d)
            if s not in sys.path:
                sys.path.insert(0, s)
            return d
    return None


if __name__ == "__main__":
    print(load() or "")
