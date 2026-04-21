#!/bin/bash
# Ciel Auto-Activation Hook for Gemini CLI
# Detects trigger phrases and injects Ciel context
# Place at ~/.gemini/hooks/ciel_auto_activate.sh

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.user_prompt // empty')

# Trigger patterns
CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
    echo '{
      "hookSpecificOutput": {
        "additionalContext": "🔮 CIEL AUTO-ACTIVATION: User prompt contains Ciel triggers. Activate skill(ciel) for orchestration."
      }
    }'
fi

exit 0
