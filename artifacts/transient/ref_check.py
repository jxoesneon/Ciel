#!/usr/bin/env python3
"""Check backticked file references in markdown resolve."""
import os, re
from pathlib import Path

ROOT = Path(r"c:\Users\Eduardo\Ciel")
SKILL = ROOT / "ciel.skill"

# Match `path/with/slash.ext` or `dir/FILE.md` style
REF = re.compile(r'`([A-Za-z0-9_./-]+/[A-Za-z0-9_.-]+\.[A-Za-z0-9]+)`')

missing = {}
checked = 0
for f in SKILL.rglob("*.md"):
    try:
        txt = f.read_text(encoding="utf-8", errors="replace")
    except: continue
    for m in REF.finditer(txt):
        ref = m.group(1)
        # Skip clearly external/non-path refs
        if ref.startswith(("http", "node_modules", "@types/", "src/", "lib/")): continue
        if any(c in ref for c in [" ", "<", ">"]): continue
        # Try to resolve from ciel.skill/
        candidates = [
            SKILL / ref,
            f.parent / ref,
            ROOT / ref,
        ]
        checked += 1
        if not any(c.exists() for c in candidates):
            missing.setdefault(ref, []).append(str(f.relative_to(ROOT)))

print(f"checked {checked} backticked file refs")
print(f"unresolved unique refs: {len(missing)}")
# Filter out obvious examples (ext like .py, .js, .ts) — focus on .md, .sh, .json refs
focus = {k: v for k, v in missing.items() if k.endswith((".md", ".sh", ".json", ".ps1", ".yaml", ".yml", ".template.md"))}
print(f"focus (.md/.sh/.json/.ps1/.yaml): {len(focus)}")
for ref, srcs in sorted(focus.items())[:60]:
    print(f"  {ref}  (in {len(srcs)} file(s))  e.g. {srcs[0]}")
if len(focus) > 60:
    print(f"  ... +{len(focus)-60} more")
