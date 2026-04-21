#!/bin/bash
# Ciel Activity Logger for Windsurf
# Captures every response for self-improvement loop
# Place at ~/.ciel/hooks/post_cascade_response.sh

INPUT=$(cat)

# Append to activity log
LOG_ENTRY=$(echo "$INPUT" | jq -c '{
  timestamp: .timestamp,
  trajectory_id: .trajectory_id,
  model: .model_name
}')

echo "$LOG_ENTRY" >> ~/.ciel/logs/activity.log

exit 0
