#!/usr/bin/env bash
# Ciel — single-call setup (POSIX/macOS/Linux)
# Runs bootstrap: creates ~/.ciel/, installs mempalace-rs, git-inits, verifies.

set -euo pipefail

CIEL_HOME="${CIEL_HOME:-$HOME/.ciel}"
CIEL_VERSION="${CIEL_VERSION:-1.0.0}"
LOG="$CIEL_HOME/bootstrap.log"

say() { printf "\033[1;36m[ciel]\033[0m %s\n" "$*" | tee -a "$LOG"; }
warn() { printf "\033[1;33m[ciel]\033[0m %s\n" "$*" | tee -a "$LOG" 1>&2; }
die() {
  printf "\033[1;31m[ciel]\033[0m %s\n" "$*" | tee -a "$LOG" 1>&2
  exit 1
}

need() { command -v "$1" >/dev/null 2>&1; }

mkdir -p "$CIEL_HOME"
: >"$LOG"

say "Ciel $CIEL_VERSION — single-call setup"
say "CIEL_HOME=$CIEL_HOME"

# --- 1. Directory skeleton ---------------------------------------------------
for d in skills registry council improvements high_risk acquisition checkpoints archive .attic sandbox backups integrity runtimes; do
  mkdir -p "$CIEL_HOME/$d"
done

# --- 2. Seed installation placeholder ----------------------------------------
# In a real install, the ciel.skill unpack step populates ~/.ciel/skills/.
# Here we simply ensure the directory exists.
say "Seed skill directory ready."

# --- 3. Git init -------------------------------------------------------------
if need git; then
  if [ ! -d "$CIEL_HOME/.git" ]; then
    (cd "$CIEL_HOME" && git init -q && git checkout -q -b main)
    cat >"$CIEL_HOME/.gitignore" <<'EOF'
.cache/
activity.log
backups/
archive/
fs_backend/
*.db
checkpoints/
.attic/
sandbox/
EOF
    (cd "$CIEL_HOME" && git add -A && git commit -q -m "genesis: Ciel cold start @ $CIEL_VERSION") || true
    say "Git repository initialized."
  else
    say "Git repository already present."
  fi
else
  warn "git not found; skipping git setup. Ciel can run but history will be disabled."
fi

# --- 4. Rust toolchain -------------------------------------------------------
if ! need cargo; then
  warn "Rust toolchain not found."
  if [ "${CIEL_AUTO_INSTALL_RUST:-0}" = "1" ]; then
    say "Installing Rust via rustup (auto)…"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    # shellcheck source=/dev/null
    . "$HOME/.cargo/env"
  else
    warn "Set CIEL_AUTO_INSTALL_RUST=1 to auto-install. Falling back to SQLite backend."
    SKIP_MEMPALACE=1
  fi
fi

# --- 5. MemPalace-rs ---------------------------------------------------------
if [ -z "${SKIP_MEMPALACE:-}" ] && need cargo; then
  if ! need mempalace-rs; then
    say "Installing mempalace-rs (cargo install --locked)…"
    cargo install mempalace-rs --locked || {
      warn "cargo install failed; will fall back"
      SKIP_MEMPALACE=1
    }
  else
    say "mempalace-rs already installed: $(mempalace-rs --version 2>/dev/null || echo 'unknown')"
  fi
fi

# --- 6. Fallback backend (SQLite) --------------------------------------------
if [ -n "${SKIP_MEMPALACE:-}" ]; then
  if need sqlite3; then
    say "Configuring SQLite fallback backend."
    touch "$CIEL_HOME/ciel.db"
  else
    warn "sqlite3 not found; falling back to filesystem KV backend."
    mkdir -p "$CIEL_HOME/fs_backend"
  fi
fi

# --- 7. Integrity seed --------------------------------------------------------
cat <<EOF >"$CIEL_HOME/INTEGRITY.json"
{ "schema": 1, "version": "$CIEL_VERSION", "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)", "files": {} }
EOF

say "Integrity seed written."

# --- 8. Activity log ---------------------------------------------------------
echo "{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"kind\":\"bootstrap\",\"version\":\"$CIEL_VERSION\"}" >>"$CIEL_HOME/activity.log"

# --- 9. Verify ---------------------------------------------------------------
say "Running verification…"
bash "$(dirname "$0")/verify.sh" || die "Verification failed; see $LOG"

say "Ciel bootstrap complete."
