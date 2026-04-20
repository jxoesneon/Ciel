#!/usr/bin/env bash
# Ciel — dependency verification + integrity check.
# Exits non-zero on any failure.

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
# Single-element iteration is intentional to leave the structure open for
# additional required files without reformatting.
# shellcheck disable=SC2043
for f in INTEGRITY.json; do
  if [ -f "$CIEL_HOME/$f" ]; then
    say "file ok: $f"
  else
    fail "file missing: $f"
  fi
done

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
if command -v mempalace-rs >/dev/null 2>&1; then
  VER="$(mempalace-rs --version 2>/dev/null || true)"
  if [ -n "$VER" ]; then
    say "mempalace-rs ok: $VER"
  else
    fail "mempalace-rs present but not runnable"
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
