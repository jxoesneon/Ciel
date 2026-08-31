---
name: devin-conversation-recovery
version: 1.0.0
format: skill/1.0
description: CIEL's framework for recovering past Devin CLI/ACP conversations from local SQLite stores. A conversation archaeology primitive for context recovery across sessions.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ops"]
triggers:
  - pattern: "(recover|find|extract|locate).*(devin|conversation|session|acp)"
    confidence: 0.9
  - pattern: "devin.*(history|transcript|archive|past)"
    confidence: 0.85
source: { tier: 2, origin: "devin-conversation-recovery" }
dependencies: { skills: [], mcp: [], system: ["python3", "sqlite3"] }
side_effects: ["fs"]
---

# CIEL ADAPTATION: Devin Conversation Recovery

This skill locates and extracts past Devin CLI/ACP conversations from local SQLite stores on disk. Devin runs as an ACP provider inside Windsurf ("Devin Local") — conversations are NOT in Hermes's session DB. This is a conversation archaeology primitive for recovering context lost between sessions. Ciel's autonomy ladder gates filesystem reads of credential-bearing stores — any secrets encountered trigger user escalation.

## Storage Layout

- **Session metadata**: `~/Library/Application Support/Devin/User/globalStorage/state.vscdb` → `ItemTable` keys `windsurf.acp.sessioninfo.session.*` and `windsurf.acp.messageStore.index` (maps sessionId → message-db uuid).
- **Message bodies**: `~/Library/Application Support/Devin/User/acp-messages/<uuid>.db` → table `messages(position, kind, payload)`; payload is JSON with streaming text chunks plus `tool_call` rows carrying `rawInput` and exit codes.

## Quick Path: Helper Script

```
python ~/.ciel/skills/devin-conversation-recovery/scripts/find_devin_convo.py --list
python .../find_devin_convo.py "blender-mcp"          # titles + first messages
python .../find_devin_convo.py "search text" --deep   # grep ALL payloads (slow)
python .../find_devin_convo.py --extract <uuid> -o /tmp/convo.txt
```

Extract output is plain text (`=== [position] kind` sections). Full transcripts can exceed 1M chars — dump to a file and page through rather than printing.

## Manual Procedure

1. **List sessions**: Query `state.vscdb` for `windsurf.acp.sessioninfo.session.%` keys; parse `messageStore.index` JSON for uuid mapping.
2. **Search bodies**: Iterate all `acp-messages/*.db` files, substring-match against `payload` column.
3. **Extract**: Payloads are JSON; `payload.content` is either a dict or a list of chunk dicts — handle both shapes. Kinds: `agent_message`, `agent_thought`, `plan`, `subagent`, `tool_call`.

## Safety Considerations

- These SQLite stores may contain credentials, API keys, or PII from past tool calls.
- Ciel's autonomy ladder escalates to the user when secrets are detected in extracted content.
- Never print raw transcripts to shared logs — always write to a local file first.
- Also check `User/workspaceStorage/*/state.vscdb` for backup copies.

## Anti-Patterns

- **Name-based search**: Session ids are whimsical names (`swanky-leaf`) — search by title or body text, not name guesses.
- **Title-only search**: Titles alone miss a lot — escalate to `--deep` body search before concluding "not found".
- **Printing huge transcripts**: Full transcripts can be 1M+ chars — always dump to file and page through.
- **Ignoring content shape variance**: `payload.content` can be a dict OR a list — code must handle both.
