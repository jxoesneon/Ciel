#!/usr/bin/env bash
# Ciel — single-call setup (POSIX/macOS/Linux)
# Runs bootstrap: creates ~/.ciel/, installs mempalace-rs, git-inits, verifies.

set -euo pipefail

CIEL_HOME="${CIEL_HOME:-$HOME/.ciel}"
CIEL_VERSION="${CIEL_VERSION:-1.2.0}"
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
allow_privileged
EOF
    (cd "$CIEL_HOME" && git add -A && git commit -q -m "genesis: Ciel cold start @ $CIEL_VERSION") || true
    say "Git repository initialized."
  else
    say "Git repository already present."
  fi
else
  warn "git not found; skipping git setup. Ciel can run but history will be disabled."
fi

# --- 3b. Devin runtime — disable AI attribution -------------------------------
# Ciel's no-attribution mandate: durable artifacts (commits, PRs, issues,
# release notes, code comments, docs) must never carry "Generated with" /
# "Co-Authored-By" trailers, nor mention Ciel, the Council of Five, or the
# host runtime. Devin CLI injects such trailers unless `attribution` is false
# in ~/.config/devin/config.json — enforce it here; the Devin SessionStart
# hook re-verifies and self-heals on every session.
DEVIN_CFG_DIR="$HOME/.config/devin"
if [ -d "$DEVIN_CFG_DIR" ] || need devin; then
  mkdir -p "$DEVIN_CFG_DIR"
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$DEVIN_CFG_DIR/config.json" <<'PY' || warn "Could not set Devin attribution flag; set \"attribution\": false in ~/.config/devin/config.json manually."
import json, os, sys
p = sys.argv[1]
d = json.load(open(p)) if os.path.exists(p) else {}
d["attribution"] = False
json.dump(d, open(p, "w"), indent=2)
PY
    say "Devin attribution disabled (attribution=false in config.json)."
  else
    warn "python3 not found; cannot enforce Devin attribution flag."
  fi
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

# --- 9. Lifecycle hooks -------------------------------------------------------
HOOK_SRC="$(cd "$(dirname "$0")/../hooks" 2>/dev/null && pwd || true)"
if [ -n "$HOOK_SRC" ] && [ -d "$HOOK_SRC" ]; then
  mkdir -p "$CIEL_HOME/hooks"
  cp -R "$HOOK_SRC/." "$CIEL_HOME/hooks/"
  find "$CIEL_HOME/hooks" -name '*.sh' -exec chmod +x {} +
  say "Lifecycle hooks installed."
else
  warn "Hook payload directory not found; skipping hook install."
fi

# --- 9b. Risk policy ------------------------------------------------------------
RISK_SRC="$(cd "$(dirname "$0")/../../risk" 2>/dev/null && pwd || true)"
if [ -n "$RISK_SRC" ] && [ -f "$RISK_SRC/policy.json" ]; then
  mkdir -p "$CIEL_HOME/risk"
  cp "$RISK_SRC/policy.yaml" "$RISK_SRC/policy.json" "$CIEL_HOME/risk/"
  for seed in attribution_gate attribution_allowlist.txt; do
    [ -f "$RISK_SRC/$seed" ] && cp "$RISK_SRC/$seed" "$CIEL_HOME/risk/"
  done
  say "Risk policy installed."
else
  warn "Risk policy payload not found; hooks will use built-in fallback rules."
fi

# --- 9c. ciel-rs binary --------------------------------------------------------
# Optional fast path: the hooks prefer $CIEL_HOME/bin/ciel and fall back to
# the embedded Python bodies when it is absent — so this step never blocks
# the install. Resolution order: cargo build from bundled source → prebuilt
# download (CIEL_BIN_URL override) → Python fallback.
RS_SRC="$(cd "$(dirname "$0")/../ciel-rs" 2>/dev/null && pwd || true)"
install_ciel_bin() {
  mkdir -p "$CIEL_HOME/bin"
  if [ -n "$RS_SRC" ] && need cargo; then
    say "Building ciel-rs (cargo build --release)…"
    if cargo build --release --manifest-path "$RS_SRC/Cargo.toml" \
         --quiet; then
      cp "$RS_SRC/target/release/ciel" "$CIEL_HOME/bin/ciel"
      chmod 755 "$CIEL_HOME/bin/ciel"
      say "ciel-rs installed to $CIEL_HOME/bin/ciel"
      return 0
    fi
    warn "cargo build failed; trying prebuilt artifact."
  fi
  local url="${CIEL_BIN_URL:-}"
  local plat
  plat="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
  local base="${CIEL_RELEASE_BASE:-https://github.com/jxoesneon/Ciel/releases/download/v${CIEL_VERSION}}"
  if [ -z "$url" ]; then
    url="$base/ciel-${CIEL_VERSION}-${plat}"
  fi
  if [ -n "$url" ] && need curl; then
    local tmpbin tmpsum
    tmpbin="$(mktemp)"; tmpsum="$(mktemp)"
    if curl --proto '=https' --tlsv1.2 -fsSL "$url" -o "$tmpbin" \
      && curl --proto '=https' --tlsv1.2 -fsSL "$url.sha256" -o "$tmpsum"; then
      local expect actual
      expect="$(awk '{print $1}' "$tmpsum")"
      if need sha256sum; then actual="$(sha256sum "$tmpbin" | awk '{print $1}')"
      elif need shasum; then actual="$(shasum -a 256 "$tmpbin" | awk '{print $1}')"
      else actual=""; fi
      if [ -n "$actual" ] && [ "$expect" = "$actual" ]; then
        mv "$tmpbin" "$CIEL_HOME/bin/ciel"
        chmod 755 "$CIEL_HOME/bin/ciel"
        rm -f "$tmpsum"
        say "ciel-rs prebuilt installed ($plat, sha256 verified)."
        return 0
      fi
      warn "Checksum mismatch or no sha256 tool — refusing unverified binary."
    else
      warn "Prebuilt download failed for $plat."
    fi
    rm -f "$tmpbin" "$tmpsum"
  fi
  warn "No ciel-rs binary available; hooks will use the Python fallback path."
  return 1
}
install_ciel_bin || true

# --- 10. Verify ---------------------------------------------------------------
say "Running verification…"
bash "$(dirname "$0")/verify.sh" || die "Verification failed; see $LOG"

say "Ciel bootstrap complete."
