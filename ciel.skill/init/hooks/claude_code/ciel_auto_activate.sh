#!/bin/bash
# Ciel Auto-Activation Hook for Claude Code
# Detects trigger phrases and forces skill activation
# Place at ~/.claude/hooks/ciel_auto_activate.sh

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.prompt // empty')

# System-1 context surfaces — detached shadow dispatch. Compaction, mandate
# canary, and context_select verdicts accrue in ~/.ciel/system1/events.jsonl
# as training signal; never blocks or gates the prompt path.
S1_LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd)/lib/system1.py"
[ -f "$S1_LIB" ] || S1_LIB="${CIEL_HOME:-$HOME/.ciel}/hooks/lib/system1.py"
if command -v python3 >/dev/null 2>&1 && [ -f "$S1_LIB" ]; then
  ( CIEL_HOOK_INPUT="$INPUT" \
      python3 "$S1_LIB" --prompt-shadow "claude_code/ciel_auto_activate" \
      >/dev/null 2>&1 & )
fi

# Trigger patterns for Ciel
CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
  # Inject skill activation instruction
  echo '{"hookSpecificOutput": {"additionalContext": "🔮 CIEL ACTIVATION: Use Skill(ciel) for orchestration. Triggers detected in user prompt."}}'
fi

exit 0
