#!/usr/bin/env python3
"""Comprehensive audit checks for Ciel project."""
import os, re, sys, json
import yaml

ROOT = r"c:\Users\Eduardo\Ciel"
TARGETS = ["ciel.skill", "skills", "agents"]
FM = re.compile(r'^---\s*\n(.*?)\n---\s*\n', re.DOTALL)

problems = {"yaml_errors": [], "missing_keys": [], "version_drift": [], "runtime_naming": [], "trigger_shape": [], "format_value": [], "license_value": []}

skill_required = {"name", "version", "description", "license", "tags"}
allowed_runtimes_canon = {"claude-code", "gemini-cli", "windsurf", "generic"}

def rel(p): return os.path.relpath(p, ROOT)

for t in TARGETS:
    base = os.path.join(ROOT, t)
    for d, _, files in os.walk(base):
        for n in files:
            if not n.endswith(".md"): continue
            path = os.path.join(d, n)
            with open(path, "r", encoding="utf-8", errors="replace") as f:
                head = f.read(8192)
            m = FM.match(head)
            if not m: continue
            try:
                data = yaml.safe_load(m.group(1)) or {}
            except yaml.YAMLError as e:
                problems["yaml_errors"].append((rel(path), str(e)))
                continue
            if not isinstance(data, dict): continue
            # Check skills/ SKILL.md keys
            if t == "skills" and n == "SKILL.md":
                missing = skill_required - data.keys()
                if missing:
                    problems["missing_keys"].append((rel(path), sorted(missing)))
                if "version" in data and str(data["version"]) != "1.0.0":
                    problems["version_drift"].append((rel(path), data["version"]))
                if "format" in data and data["format"] != "skill/1.0":
                    problems["format_value"].append((rel(path), data["format"]))
                if "license" in data and data["license"] != "MIT":
                    problems["license_value"].append((rel(path), data["license"]))
                # runtimes naming
                rts = data.get("runtimes")
                if isinstance(rts, list):
                    for r in rts:
                        if r not in allowed_runtimes_canon:
                            problems["runtime_naming"].append((rel(path), r))
                # triggers shape
                trg = data.get("triggers")
                if trg is not None and not isinstance(trg, list):
                    problems["trigger_shape"].append((rel(path), "not list"))

# Print
for k, v in problems.items():
    print(f"=== {k}: {len(v)} ===")
    for item in v[:30]:
        print("  ", item)
    if len(v) > 30:
        print(f"  ... +{len(v)-30} more")
