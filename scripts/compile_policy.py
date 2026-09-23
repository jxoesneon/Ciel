#!/usr/bin/env python3
"""Compile ciel.skill/risk/policy.yaml -> risk/policy.json.

policy.yaml is the human-edited source of truth; the JSON twin is what the
PreToolUse hooks load so the evaluation path stays stdlib-only. Run without
arguments to regenerate; run with --check to verify the JSON is in sync
(used by CI and the local lint gate).
"""

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
POLICY_DIR = ROOT / "ciel.skill" / "risk"
YAML_PATH = POLICY_DIR / "policy.yaml"
JSON_PATH = POLICY_DIR / "policy.json"


def compiled() -> dict:
    try:
        import yaml
    except ImportError:
        sys.exit("compile_policy.py requires PyYAML (dev-time dependency)")
    data = yaml.safe_load(YAML_PATH.read_text(encoding="utf-8"))
    if not isinstance(data, dict) or not isinstance(data.get("rules"), list):
        sys.exit("policy.yaml: expected a mapping with a 'rules' list")
    return data


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="verify policy.json matches policy.yaml")
    args = parser.parse_args()

    data = compiled()
    rendered = json.dumps(data, indent=2, ensure_ascii=False) + "\n"

    if args.check:
        if not JSON_PATH.is_file():
            print("[policy] policy.json missing; run scripts/compile_policy.py")
            return 1
        if JSON_PATH.read_text(encoding="utf-8") != rendered:
            print("[policy] policy.json is stale; run scripts/compile_policy.py")
            return 1
        print(f"[policy] policy.json in sync ({len(data['rules'])} rules)")
        return 0

    JSON_PATH.write_text(rendered, encoding="utf-8")
    print(f"[policy] wrote {JSON_PATH} ({len(data['rules'])} rules)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
