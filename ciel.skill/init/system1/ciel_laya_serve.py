#!/usr/bin/env python3
"""ciel_laya_serve — laya-serve entrypoint with a local-checkpoint registry overlay.

laya-serve accepts model names only through ``laya.router.normalise_name``,
which validates against the hardcoded ``DEFAULT_MODELS`` registry, so a
locally trained sibling checkpoint (e.g. a retrained head written next to
the served one) can never be served. This entrypoint registers named local
checkpoints before ``build_router()`` runs, then hands off to laya's own
server unchanged — one transport, no fork.

Overlay file: ``CIEL_LAYA_MODELS_FILE`` or ``~/.ciel/system1/models.local.json``::

    {"<name>": {"path": "<absolute checkpoint dir>",
                "sha256": {"<artifact>": "<hex digest>", ...}}}

Fail-closed rules (council binding conditions):

* the file must be owner-only (no group/world bits) — write access to it is
  serving control;
* each name must match ``[a-z0-9][a-z0-9_-]{1,63}`` and not collide with a
  bundled ``DEFAULT_MODELS`` name or ``_ALIASES`` entry;
* each entry requires an absolute, existing checkpoint directory and a
  non-empty sha256 map — no overlay row ever loads unverified bytes;
* any violation refuses startup rather than partially registering.

Routing facts relied on (laya/router.py ``route()``): overlay names are
reachable only via an explicit request ``model=`` — auto-routing emits only
``english``, ``multilingual``, ``typed-decisions`` or the configured
``default``, and workflow auto-detection hardcodes ``typed-decisions``.
Preload stays governed by ``LAYA_MODELS``.

This shim is the interim seam; the durable fix is an upstream laya
``LAYA_EXTRA_MODELS``-style registry extension.
"""

import json
import os
import re
import stat
import sys
from pathlib import Path

NAME_RE = re.compile(r"^[a-z0-9][a-z0-9_-]{1,63}$")


def _overlay_path() -> Path:
    raw = os.environ.get("CIEL_LAYA_MODELS_FILE")
    if raw:
        return Path(raw)
    home = os.environ.get("CIEL_HOME") or str(Path.home() / ".ciel")
    return Path(home) / "system1" / "models.local.json"


def _die(msg: str) -> "SystemExit":
    return SystemExit(f"[ciel_laya_serve] {msg}")


def load_overlay(path: Path) -> dict:
    """Validate the overlay file and return {name: {"path", "sha256"}}.

    Returns {} when the file is absent. Any malformed content raises
    SystemExit — a half-valid registry is a configuration error, not a
    degraded state.
    """
    if not path.is_file():
        return {}

    mode = stat.S_IMODE(path.stat().st_mode)
    if mode & 0o077:
        raise _die(f"refusing {path}: group/world permission bits set "
                   f"({oct(mode)}) — the overlay controls what the endpoint "
                   "serves; chmod 600 it")

    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        raise _die(f"refusing {path}: unreadable JSON ({e})")
    if not isinstance(data, dict):
        raise _die(f"refusing {path}: top level must be an object")

    import laya.router as lr  # deferred: cheap import until needed

    entries = {}
    for name, spec in data.items():
        if not isinstance(name, str) or not NAME_RE.match(name):
            raise _die(f"invalid model name {name!r}")
        if name in lr.DEFAULT_MODELS or name in lr._ALIASES:
            raise _die(f"overlay name {name!r} collides with the bundled "
                       "registry — refusing to shadow a built-in")
        if not isinstance(spec, dict):
            raise _die(f"entry {name!r}: spec must be an object")
        ckpt = spec.get("path")
        digests = spec.get("sha256")
        if not isinstance(ckpt, str) or not Path(ckpt).is_absolute():
            raise _die(f"entry {name!r}: 'path' must be an absolute path")
        if not Path(ckpt).is_dir():
            raise _die(f"entry {name!r}: checkpoint dir does not exist: {ckpt}")
        if (not isinstance(digests, dict) or not digests
                or not all(isinstance(k, str) and isinstance(v, str)
                           for k, v in digests.items())):
            raise _die(f"entry {name!r}: 'sha256' must be a non-empty "
                       "{artifact: hex-digest} map — overlay checkpoints are "
                       "never served unverified")
        entries[name] = {"path": ckpt, "sha256": dict(digests)}
    return entries


def apply_overlay(entries: dict) -> None:
    """Register overlay names in laya's registry tables and pin digests via
    LAYA_SHA256_DIGESTS, which Router._digests_from_env merges per model."""
    import laya.router as lr

    for name, spec in entries.items():
        lr.DEFAULT_MODELS[name] = (spec["path"], None)

    existing_raw = os.environ.get("LAYA_SHA256_DIGESTS", "").strip()
    merged = {}
    if existing_raw:
        try:
            existing = json.loads(existing_raw)
        except ValueError:
            existing = None
        if isinstance(existing, dict) and all(
                isinstance(v, dict) for v in existing.values()):
            merged = {k: dict(v) for k, v in existing.items()}
        # a flat {artifact: digest} map or unparseable value is left to
        # laya.revisions' own error path; we do not merge into it
    for name, spec in entries.items():
        merged[name] = spec["sha256"]
    os.environ["LAYA_SHA256_DIGESTS"] = json.dumps(merged)


def main() -> None:
    path = _overlay_path()
    entries = load_overlay(path)
    if entries:
        apply_overlay(entries)
        print(f"[ciel_laya_serve] registered {sorted(entries)} from {path}")
        if not os.environ.get("LAYA_MODELS", "").strip():
            print("[ciel_laya_serve] warning: LAYA_MODELS unset — a "
                  "names=None preload would build overlay checkpoints too",
                  file=sys.stderr)
    import laya.serve
    laya.serve.main()


if __name__ == "__main__":
    main()
