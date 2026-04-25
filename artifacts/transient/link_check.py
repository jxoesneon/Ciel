#!/usr/bin/env python3
"""Check internal markdown links resolve to existing files."""
import os, re, sys
from pathlib import Path

ROOT = Path(r"c:\Users\Eduardo\Ciel")
LINK = re.compile(r'(?<!!)\[([^\]]+)\]\(([^)]+)\)')

broken = []
checked = 0
for base in ["ciel.skill", "skills", "agents", "README.md", "CONTRIBUTING.md"]:
    p = ROOT / base
    files = [p] if p.is_file() else list(p.rglob("*.md"))
    for f in files:
        try:
            txt = f.read_text(encoding="utf-8", errors="replace")
        except Exception:
            continue
        for m in LINK.finditer(txt):
            target = m.group(2).split("#")[0].strip()
            if not target: continue
            if target.startswith(("http://", "https://", "mailto:", "ftp://", "data:")): continue
            if target.startswith("/"): continue  # workspace-relative; skip
            checked += 1
            resolved = (f.parent / target).resolve()
            if not resolved.exists():
                broken.append((str(f.relative_to(ROOT)), target))

print(f"checked {checked} relative md links")
print(f"broken: {len(broken)}")
for src, tgt in broken[:60]:
    print(f"  {src} -> {tgt}")
if len(broken) > 60:
    print(f"  ... +{len(broken)-60} more")
