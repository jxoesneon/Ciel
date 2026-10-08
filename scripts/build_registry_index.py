#!/usr/bin/env python3
"""Build ~/.ciel/registry/index.json from the filesystem source of truth.

Scans CIEL_SKILLS_DIR (default ~/.ciel/skills/) for */SKILL.md, merges
frontmatter with the sibling ciel.yaml extension when present, and emits the
index atomically (tmp write + rename) per registry/INDEXING.md.

Usage: build_registry_index.py [--skills-dir DIR] [--out PATH]
"""

import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

HOME = Path.home()
SKILLS_DIR = Path(os.environ.get("CIEL_SKILLS_DIR", HOME / ".ciel" / "skills"))
INDEX_PATH = HOME / ".ciel" / "registry" / "index.json"


def _load_yaml(text: str) -> dict:
    """Minimal YAML reader sufficient for ciel.yaml/frontmatter shapes.

    Tries PyYAML first (present in system1 venv); falls back to a tiny
    indentation parser that handles flat keys, nested maps, and scalar/
    inline lists — enough for skill metadata, not general YAML.
    """
    try:
        import yaml  # type: ignore
        return yaml.safe_load(text) or {}
    except ImportError:
        pass

    def parse_block(lines, i, indent):  # noqa: PLR0912
        out = {}
        while i < len(lines):
            line = lines[i]
            if not line.strip() or line.lstrip().startswith("#"):
                i += 1
                continue
            cur = len(line) - len(line.lstrip())
            if cur < indent:
                break
            if cur > indent:
                i += 1
                continue
            m = re.match(r"([^:]+):\s*(.*)$", line.strip())
            if not m:
                i += 1
                continue
            key, val = m.group(1).strip(), m.group(2).strip()
            if val == "":
                # nested map or list
                j = i + 1
                items = []
                submap = None
                while j < len(lines):
                    nl = lines[j]
                    if not nl.strip():
                        j += 1
                        continue
                    ni = len(nl) - len(nl.lstrip())
                    if ni <= indent:
                        break
                    if nl.strip().startswith("- "):
                        items.append(nl.strip()[2:].strip())
                        j += 1
                    else:
                        submap, j = parse_block(lines, j, ni)
                        break
                out[key] = submap if submap is not None else items
                i = j
            elif val.startswith("[") and val.endswith("]"):
                out[key] = [v.strip() for v in val[1:-1].split(",") if v.strip()]
                i += 1
            else:
                out[key] = val.strip("'\"")
                i += 1
        return out, i

    result, _ = parse_block(text.splitlines(), 0, 0)
    return result


def _frontmatter(path: Path) -> dict:
    text = path.read_text(encoding="utf-8", errors="replace")
    if not text.startswith("---"):
        return {}
    end = text.find("\n---", 3)
    if end == -1:
        return {}
    return _load_yaml(text[3:end])


def _checksum(path: Path) -> str:
    h = hashlib.sha256()
    for f in sorted(path.rglob("*")):
        if f.is_file():
            h.update(str(f.relative_to(path)).encode())
            h.update(f.read_bytes())
    return "sha256:" + h.hexdigest()


def build_entry(skill_dir: Path) -> dict | None:
    skill_md = skill_dir / "SKILL.md"
    if not skill_md.is_file():
        return None
    fm = _frontmatter(skill_md)
    ciel = skill_dir / "ciel.yaml"
    ext = _load_yaml(ciel.read_text(encoding="utf-8", errors="replace")) if ciel.is_file() else {}

    triggers = ext.get("triggers") or []
    trig_patterns = [
        t.get("pattern", "") if isinstance(t, dict) else str(t) for t in triggers
    ]

    stat = skill_md.stat()
    entry = {
        "id": skill_dir.name,
        "version": str(ext.get("version") or fm.get("version") or "0.0.0"),
        "description": fm.get("description", ""),
        "triggers": trig_patterns,
        "tags": ext.get("tags") or fm.get("tags") or [],
        "license": fm.get("license") or (ext.get("source") or {}).get("license"),
        "source": ext.get("source") or {"tier": 0, "origin": "local"},
        "install_path": str(skill_dir) + "/",
        "state": ext.get("state", "validated"),
        "checksum": _checksum(skill_dir),
        "created": datetime.fromtimestamp(stat.st_ctime, timezone.utc).isoformat(),
        "last_updated": datetime.fromtimestamp(stat.st_mtime, timezone.utc).isoformat(),
    }
    for opt in ("dependencies", "side_effects", "runtimes"):
        if opt in ext:
            entry[opt] = ext[opt]
    return entry


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--skills-dir", type=Path, default=SKILLS_DIR)
    ap.add_argument("--out", type=Path, default=INDEX_PATH)
    args = ap.parse_args()

    entries = {}
    skipped = []
    for d in sorted(args.skills_dir.iterdir()):
        if not d.is_dir():
            continue
        e = build_entry(d)
        if e is None:
            skipped.append(d.name)
        else:
            entries[e["id"]] = e

    index = {
        "schema": 1,
        "built": datetime.now(timezone.utc).isoformat(),
        "skills_dir": str(args.skills_dir),
        "count": len(entries),
        "skills": entries,
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(dir=args.out.parent, suffix=".tmp")
    with os.fdopen(fd, "w") as f:
        json.dump(index, f, indent=1, ensure_ascii=False)
    os.rename(tmp, args.out)

    print(json.dumps({
        "indexed": len(entries),
        "skipped_no_skill_md": skipped,
        "out": str(args.out),
    }, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
