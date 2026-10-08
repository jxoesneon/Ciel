#!/bin/bash
# Ciel Auto-Activation Hook for Windsurf
# Detects trigger phrases in user prompt
# Place at ~/.ciel/hooks/pre_user_prompt.sh

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.tool_info.user_prompt // empty')

# System-1 context surfaces — detached shadow dispatch. Compaction, mandate
# canary, and context_select verdicts accrue in ~/.ciel/system1/events.jsonl
# as training signal; never blocks or gates the prompt path.
S1_LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd)/lib/system1.py"
[ -f "$S1_LIB" ] || S1_LIB="${CIEL_HOME:-$HOME/.ciel}/hooks/lib/system1.py"
if command -v python3 >/dev/null 2>&1 && [ -f "$S1_LIB" ]; then
  (CIEL_HOOK_INPUT="$INPUT" \
    python3 "$S1_LIB" --prompt-shadow "windsurf/pre_user_prompt" \
    >/dev/null 2>&1 &)
fi

# Trigger patterns
CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
  # Output to stderr (shown to user if show_output: true)
  echo "🔮 CIEL ACTIVATION: Trigger phrases detected" >&2
fi

exit 0
