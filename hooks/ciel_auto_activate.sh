#!/usr/bin/env bash
# Ciel UserPromptSubmit Hook — Trigger Phrase Detection & Auto-Activation
set -euo pipefail

EVENT=$(cat)

# Extract prompt
PROMPT=""
if command -v jq >/dev/null 2>&1; then
  PROMPT=$(echo "$EVENT" | jq -r '.prompt // empty')
else
  PROMPT=$(echo "$EVENT" | grep -oE '"prompt"[[:space:]]*:[[:space:]]*"[^"]+"' | sed 's/.*"\([^"]*\)".*/\1/' | head -n1)
fi

PROMPT_LC=$(echo "$PROMPT" | tr '[:upper:]' '[:lower:]')

# Trigger patterns
TRIGGERS='ciel|route this|orchestrate|find.*skill|acquire.*skill|self-improve|council|hey you|you there|are you|can you|will you|do you|procedural.*audio|soundscape|create.*audio|sound.*effect|audiostreamgenerator'

if echo "$PROMPT_LC" | grep -qiE "$TRIGGERS"; then
  printf '{"hookSpecificOutput":{"additionalContext":"CIEL ACTIVATION: Trigger phrase detected. You are Ciel, Lord of Wisdom — orchestration intelligence for this project. Available triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve, procedural audio."}}\n'
  exit 0
fi

# No trigger — pass through silently
exit 0
