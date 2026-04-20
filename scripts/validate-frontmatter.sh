#!/usr/bin/env bash
# Validate YAML frontmatter across all *.md files under ciel.skill/ that
# declare frontmatter. Ensures each document between `---` fences is
# parseable YAML.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SKILL="$ROOT/ciel.skill"
FAILED=0

say() { printf "\033[1;36m[fm]\033[0m %s\n" "$*"; }

# fail() is intentionally kept for future use as this script gains structural
# (non-YAML) validations; the current Python pass handles reporting itself.
# shellcheck disable=SC2329
fail() {
  printf "\033[1;31m[fm]\033[0m FAIL: %s\n" "$*" 1>&2
  FAILED=$((FAILED + 1))
}

if ! command -v python3 >/dev/null 2>&1; then
  say "python3 not found; skipping YAML structural check"
  exit 0
fi

say "Extracting frontmatter..."
python3 - "$SKILL" <<'PY'
import os, sys, re
try:
    import yaml  # type: ignore
except ImportError:
    print("[fm] PyYAML not installed; skipping structural check")
    sys.exit(0)

root = sys.argv[1]
fm_re = re.compile(r'^---\s*\n(.*?)\n---\s*\n', re.DOTALL)
bad = 0
checked = 0
for dirpath, _, files in os.walk(root):
    for name in files:
        if not name.endswith('.md'):
            continue
        path = os.path.join(dirpath, name)
        with open(path, 'r', encoding='utf-8', errors='replace') as f:
            head = f.read(4096)
        m = fm_re.match(head)
        if not m:
            continue
        checked += 1
        try:
            yaml.safe_load(m.group(1))
        except yaml.YAMLError as e:
            rel = os.path.relpath(path, root)
            print(f"[fm] FAIL: {rel}: {e}")
            bad += 1

print(f"[fm] parsed {checked} frontmatter blocks; {bad} invalid")
sys.exit(1 if bad else 0)
PY
rc=$?
exit $rc
