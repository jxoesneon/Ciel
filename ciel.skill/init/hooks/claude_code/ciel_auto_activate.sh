#!/bin/bash
# Ciel Auto-Activation Hook for Claude Code
# Detects trigger phrases and forces skill activation
# Place at ~/.claude/hooks/ciel_auto_activate.sh

INPUT=$(cat)
PROMPT=$(echo "$INPUT" | jq -r '.prompt // empty')

# Trigger patterns for Ciel
CIEL_TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council'

if echo "$PROMPT" | grep -qiE "$CIEL_TRIGGERS"; then
    # Inject skill activation instruction
    echo '{"hookSpecificOutput": {"additionalContext": "🔮 CIEL ACTIVATION: Use Skill(ciel) for orchestration. Triggers detected in user prompt."}}'
fi

exit 0
