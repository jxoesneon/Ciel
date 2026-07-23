#!/usr/bin/env bash
# Ciel — dependency verification + integrity check.
# Exits non-zero on any failure.
# In Obsidian mode, the primary memory backend is the Obsidian vault adapter.

set -euo pipefail

CIEL_HOME="${CIEL_HOME:-$HOME/.ciel}"
FAILED=0

say() { printf "\033[1;36m[ciel-verify]\033[0m %s\n" "$*"; }
fail() {
  printf "\033[1;31m[ciel-verify]\033[0m %s\n" "$*" 1>&2
  FAILED=1
}

[ -d "$CIEL_HOME" ] || {
  fail "CIEL_HOME $CIEL_HOME does not exist"
  exit 1
}

# 1. Required directories
for d in skills registry council acquisition checkpoints; do
  if [ -d "$CIEL_HOME/$d" ]; then
    say "dir ok: $d"
  else
    fail "dir missing: $d"
  fi
done

# 2. Required files
if [ -f "$CIEL_HOME/INTEGRITY.json" ]; then
  say "file ok: INTEGRITY.json"
else
  fail "file missing: INTEGRITY.json"
fi

# 3. Git
if command -v git >/dev/null 2>&1; then
  if [ -d "$CIEL_HOME/.git" ]; then
    HEAD_SHA="$(git -C "$CIEL_HOME" rev-parse HEAD 2>/dev/null || true)"
    if [ -n "$HEAD_SHA" ]; then
      say "git ok: HEAD=$HEAD_SHA"
    else
      fail "git repo present but no HEAD"
    fi
  else
    fail "git repo missing in $CIEL_HOME"
  fi
else
  say "git not installed — history disabled (acceptable)."
fi

# 4. Memory backend
OBSIDIAN_BACKEND_DIR="$(cd "$(dirname "$0")" && pwd)/../../memory/backends/obsidian"
if command -v node >/dev/null 2>&1 && [ -f "$OBSIDIAN_BACKEND_DIR/cli.mjs" ]; then
  if node "$OBSIDIAN_BACKEND_DIR/cli.mjs" --self-test >/dev/null 2>&1; then
    say "obsidian backend ok"
  else
    warn "obsidian backend self-test failed; vault may not be open or Local REST API not enabled"
  fi
elif [ -f "$CIEL_HOME/ciel.db" ]; then
  say "sqlite fallback detected."
elif [ -d "$CIEL_HOME/fs_backend" ]; then
  say "filesystem fallback detected."
else
  fail "no memory backend present"
fi

# 5. Shell essentials
for cmd in bash grep awk sed; do
  if command -v "$cmd" >/dev/null 2>&1; then
    say "cmd ok: $cmd"
  else
    fail "cmd missing: $cmd"
  fi
done

# 6. Activity log writable
if touch "$CIEL_HOME/activity.log" 2>/dev/null; then
  say "activity.log writable"
else
  fail "activity.log not writable"
fi

if [ "$FAILED" -ne 0 ]; then
  printf "\033[1;31m[ciel-verify]\033[0m one or more checks failed\n" 1>&2
  exit 1
fi

say "all verification checks passed"
exit 0
