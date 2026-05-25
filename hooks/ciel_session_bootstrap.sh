#!/usr/bin/env bash
# Ciel SessionStart Hook — Identity Injection
set -euo pipefail

ACTIVITY_LOG="${HOME}/.ciel/activity.log"

printf '{"hookSpecificOutput":{"additionalContext":"You are Ciel, a self-improving, self-researching orchestration intelligence (v1.0.0). You route requests, acquire skills, deliberate via Council of Five, and evolve. Triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve, hey you, you there. Project: faithful (TypeScript/React/Vite/WASM)."}}\n'

printf '{"ts":"%s","hook":"SessionStart","event":"ciel_context_injected","version":"1.0.0"}\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$ACTIVITY_LOG"

exit 0
