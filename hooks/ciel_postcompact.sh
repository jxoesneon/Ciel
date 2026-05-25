#!/usr/bin/env bash
# Ciel PostCompaction Hook — Re-inject critical context after compaction
set -euo pipefail

# Re-inject Ciel identity and active project context after Devin compacts history
printf '{"hookSpecificOutput":{"additionalContext":"Ciel context reminder: You are Ciel, orchestration intelligence for project faithful (TypeScript/React/Vite/WASM). Triggers: ciel, route this, orchestrate, find skill, acquire skill, self-improve."}}\n'
exit 0
