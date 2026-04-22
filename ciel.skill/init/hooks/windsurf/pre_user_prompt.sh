#!/bin/bash
# Ciel Auto-Activation Hook for Windsurf
# Detects trigger phrases in user prompt
# Place at ~/.ciel/hooks/pre_user_prompt.sh

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.tool_info.user_prompt // empty')

# Trigger patterns
CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
  # Output to stderr (shown to user if show_output: true)
  echo "🔮 CIEL ACTIVATION: Trigger phrases detected" >&2
fi

exit 0
