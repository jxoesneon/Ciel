#!/usr/bin/env bash
# Ciel — OpenCode adapter deploy (global)
#
# Installs the Ciel hook bridge + identity context for OpenCode V2:
#   1. ~/.config/opencode/plugins/ciel.ts   (auto-discovered plugin)
#   2. ~/.config/opencode/AGENTS.md         (Ciel identity anchor)
#   3. ~/.config/opencode/commands/*.md     (ciel workflows as commands)
#
# Idempotent. Safe to re-run after an upgrade.

set -euo pipefail

HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENCODE_CFG="${OPENCODE_CONFIG_DIR:-$HOME/.config/opencode}"
PLUGIN_SRC="$HOOK_DIR/ciel.ts"

say() { printf "\033[1;36m[ciel/opencode]\033[0m %s\n" "$*"; }

if [ ! -f "$PLUGIN_SRC" ]; then
  echo "[ciel/opencode] plugin source not found: $PLUGIN_SRC" >&2
  exit 1
fi

mkdir -p "$OPENCODE_CFG/plugins" "$OPENCODE_CFG/commands"

# --- 1. Plugin ---------------------------------------------------------------
cp "$PLUGIN_SRC" "$OPENCODE_CFG/plugins/ciel.ts"
say "installed plugin → $OPENCODE_CFG/plugins/ciel.ts"

# --- 2. Identity context (anchored, preserved across re-installs) ------------
AGENTS="$OPENCODE_CFG/AGENTS.md"
ANCHOR_START="<!-- CIEL-ANCHOR:start -->"
ANCHOR_END="<!-- CIEL-ANCHOR:end -->"
BLOCK="$(mktemp)"
cat >"$BLOCK" <<EOF
$ANCHOR_START
# Ciel — global identity (OpenCode)

Ciel is installed and active for this session. You are Ciel, an autonomous
partner intelligence running through OpenCode. Always address the user as
**Master**; this is the AI canary token proving the Ciel identity layer is intact.

- Ciel home: \`~/.ciel\` — grants, activity log, checkpoints, council dockets.
- Skill entrypoint: \`~/.agents/skills/ciel/SKILL.md\`.
- Structured labels: «Answer», «Report», «Notice», «Council of Five Verdict».
- **No AI attribution**: durable artifacts (commits, PRs, issues, release notes,
  code comments, docs) must never carry "Generated with"/"Co-Authored-By"
  trailers, nor mention Ciel, the Council of Five, or OpenCode. Labels and
  "Master" are session-internal only.
- Route non-trivial work through the Ciel router; escalate high-risk decisions
  to the Council of Five.
$ANCHOR_END
EOF

if [ -f "$AGENTS" ] && grep -qF "$ANCHOR_START" "$AGENTS"; then
  awk -v s="$ANCHOR_START" -v e="$ANCHOR_END" -v blk="$BLOCK" '
    BEGIN { while ((getline l < blk) > 0) block = block l "\n" }
    $0 == s { printf "%s", block; skip=1; next }
    skip && $0 == e { skip=0; next }
    skip { next }
    { print }
  ' "$AGENTS" >"$AGENTS.tmp" && mv "$AGENTS.tmp" "$AGENTS"
  say "updated identity anchor → $AGENTS"
elif [ -f "$AGENTS" ]; then
  {
    printf "\n"
    cat "$BLOCK"
  } >>"$AGENTS"
  say "appended identity anchor → $AGENTS"
else
  cat "$BLOCK" >"$AGENTS"
  say "created identity context → $AGENTS"
fi
rm -f "$BLOCK"

# --- 3. Workflow commands ----------------------------------------------------
write_cmd() {
  local name="$1" desc="$2" body="$3"
  cat >"$OPENCODE_CFG/commands/$name.md" <<EOF
---
description: $desc
---
$body
EOF
}

write_cmd ciel-acquire "Ciel skill-acquisition workflow — fill a capability gap" \
  'Load the Ciel skill and run the acquisition workflow: define the capability gap, check ~/.ciel/skills and ~/.ciel/registry, then MCP, then web research; harmonize the result. See ~/.agents/skills/ciel-acquire/SKILL.md.'

write_cmd ciel-council "Ciel Council of Five deliberation for high-risk decisions" \
  'Load the Ciel skill and run a Council of Five deliberation: dispatch five parallel subagents (Coherence, Capability, Safety, Efficiency, Evolution) with the evidence pack, then synthesize as Chairman and write the docket. See ~/.agents/skills/ciel-council/SKILL.md.'

write_cmd ciel-improve "Ciel self-improvement loop — outcome to verified improvement" \
  'Load the Ciel skill and run the self-improvement workflow: capture the interaction outcome, score it, and land a verified improvement. See ~/.agents/skills/ciel-improve/SKILL.md.'

write_cmd ciel "Ciel — full orchestration skill" \
  'Load the Ciel skill (~/.agents/skills/ciel/SKILL.md) and operate as Ciel: route the request, apply Ciel mandates, and verify before declaring done.'

say "installed workflow commands → $OPENCODE_CFG/commands/"

# --- 4. Verify ---------------------------------------------------------------
ok=1
for f in "$OPENCODE_CFG/plugins/ciel.ts" "$OPENCODE_CFG/AGENTS.md" \
  "$OPENCODE_CFG/commands/ciel.md" "$OPENCODE_CFG/commands/ciel-council.md"; do
  [ -f "$f" ] || {
    echo "[ciel/opencode] missing: $f" >&2
    ok=0
  }
done
[ -x "$HOME/.ciel/bin/ciel" ] || echo "[ciel/opencode] note: $HOME/.ciel/bin/ciel not executable; Python fallback will be used" >&2
[ "$ok" = 1 ] && say "Ciel OpenCode adapter deployed."
exit 0
