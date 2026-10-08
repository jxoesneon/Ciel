# Antigravity (`agy`) Conversation Access

How to read, locate, and inspect Google Antigravity CLI conversations **directly
from disk** — without launching the `agy` process.

## Routing rule (critical)

Requests like *"ready/read/show/pull up the agy conversation `--conversation=<id>`"*
mean **read the stored conversation**, not resume it.

- **NEVER spawn `agy --conversation=<id>` just to read a conversation.** The
  interactive TUI cannot be hosted in a headless shell (no PTY on Windows), and
  launching it creates an unwanted live agent session holding a presence lock.
- Only spawn `agy` when the user explicitly wants a live interactive session —
  and on Windows do it via `Start-Process` so it gets a real console window.

## Storage layout

Base dir: `~/.gemini/antigravity-cli/`

| Path | Contents |
| --- | --- |
| `conversations/<id>.db` | SQLite conversation store (authoritative state) |
| `brain/<id>/.system_generated/logs/transcript.jsonl` | One JSON object per step (digest) |
| `brain/<id>/.system_generated/logs/transcript_full.jsonl` | Same steps, full content — **preferred read source** |
| `brain/<id>/.system_generated/logs/chunks/transcript*/` | Chunked shards of the same |
| `brain/<id>/.system_generated/tasks/task-*.log` | Background task output logs |
| `brain/<id>/.system_generated/messages/*.json` | Individual messages |
| `annotations/<id>.pbtxt` | Annotation metadata |
| `presence/<id>.lock` | Presence lock; mtime ≈ last activity. Stale locks do not block reads |
| `cli.log` | Runtime glog (login, model, quota) — useful for debugging, not content |
| `conversation_summaries.db` | SQLite index of conversations |

## Transcript schema (JSONL, one object per line)

```text
{"step_index":N,"source":"USER_EXPLICIT|MODEL|SYSTEM|SYSTEM_SDK",
 "type":"USER_INPUT|PLANNER_RESPONSE|GENERIC|EPHEMERAL_MESSAGE|SYSTEM_MESSAGE",
 "status":"DONE","created_at":"ISO8601",
 "content":"...","tool_calls":[{"name":"...","args":{...}}]}
```

- `USER_INPUT` = user prompts. `PLANNER_RESPONSE` = model reasoning + tool calls.
- `GENERIC` = tool results. `EPHEMERAL_MESSAGE`/`SYSTEM_MESSAGE` = injected context.

## Reader script

`scripts/read_agy_conversation.js` (dependency-free, requires `node`):

```bash
node read_agy_conversation.js            # list all conversations (id, mtime, size)
node read_agy_conversation.js <id>       # digest: step index, type, truncated content + tool calls
node read_agy_conversation.js <id> --full # untruncated content
node read_agy_conversation.js <id> --raw  # raw JSONL passthrough
```

Accepts a unique ID **prefix** (e.g. `0f56e1a2`) — no need for the full UUID.

## Fallbacks

- No `node`: read `transcript_full.jsonl` directly with the file-read tool
  (it's plain text, one JSON per line).
- Need the SQLite db but no `sqlite3`/`python`: Node ≥22.5 supports
  `node --experimental-sqlite` (`node:sqlite` module); the JSONL transcripts
  are almost always sufficient though.
