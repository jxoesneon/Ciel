---
name: devin-conversation-recovery
description: Locate and extract past Devin CLI/ACP conversations from local storage.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# Devin Conversation Recovery

Devin runs as an ACP provider inside Windsurf ("Devin Local"). Conversations are
NOT in Hermes's session DB — they live in two SQLite stores on disk:

- **Titles/session metadata**: `~/Library/Application Support/Devin/User/globalStorage/state.vscdb`

  → `ItemTable` keys `windsurf.acp.sessioninfo.session.*` and the
  `windsurf.acp.messageStore.index` key mapping sessionId → message-db uuid.

- **Message bodies**: `~/Library/Application Support/Devin/User/acp-messages/<uuid>.db`

  → table `messages(position, kind, payload)`; payload is JSON with content as
  streaming text chunks, plus `tool_call` rows carrying `rawInput` (commands,
  file reads) and exit codes.

## Quick path: use the helper script

```text
python ~/.ciel/skills/devin-conversation-recovery/scripts/find_devin_convo.py --list
python .../find_devin_convo.py "blender-mcp"          # titles + first messages
python .../find_devin_convo.py "search text" --deep   # grep ALL payloads (slow)
python .../find_devin_convo.py --extract <uuid> -o /tmp/convo.txt
```

The extract output is plain text (`=== [position] kind` sections): agent thoughts/
messages joined from streaming chunks, tool commands with `$ cmd (exit N)`, and
truncated tool output. Full transcripts can be 1M+ chars — dump to a file and
page through with read_file rather than printing.

## Manual procedure (if script unavailable)

1. List sessions:

```python
import sqlite3, json
con = sqlite3.connect("~/Library/Application Support/Devin/User/globalStorage/state.vscdb")
rows = con.execute("select key,value from ItemTable where key like 'windsurf.acp.sessioninfo.session.%'").fetchall()
idx = json.loads(con.execute("select value from ItemTable where key='windsurf.acp.messageStore.index'").fetchone()[0])

# idx["acp/devin-cli/<name>"]["uuid"] gives the acp-messages db filename

```

1. Search bodies across all dbs: iterate `User/acp-messages/*.db`,

   `select payload from messages` and substring-match.

2. Extract: payloads are JSON; user prompts are NOT their own kind — they appear

   only inside agent_thought chunks quoting them. Kinds seen: agent_message,
   agent_thought, plan, subagent, tool_call.

## Pitfalls

- Session ids are whimsical names (`swanky-leaf`) — search by title or body text, not name guesses.
- Titles alone miss a lot; if no hit, escalate to `--deep` before concluding "not found".
- `payload.content` is either a dict or a list of chunk dicts — handle both shapes.
- Archived sessions still have their dbs; only the index entry may be stale.
- Also check `User/workspaceStorage/*/state.vscdb` copies (backups exist there too).
