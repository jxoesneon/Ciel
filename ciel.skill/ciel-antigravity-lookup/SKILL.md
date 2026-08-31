---
name: ciel-antigravity-lookup
version: 1.0.1
description: Find Antigravity conversations on macOS by UUID or keyword.
author: Ciel Project
triggers: ["antigravity conversation", "antigravity chat", "find antigravity", "read antigravity", "conversation uuid"]
tags: ["ciel", "domain:devtools", "antigravity", "sqlite", "recovery"]
metadata:
  hermes:
    tags: [antigravity, sqlite, macos, recovery]
    related_skills: [session_search]
runtimes: ["generic"]
license: MIT
source:
  tier: 0
  origin: seed
dependencies:
  skills: []
  mcp: []
  system: ["sqlite3", "python3"]
---

# Antigravity Conversation Lookup

## When to Use

Load this skill whenever a conversation that happened inside the Google Antigravity IDE
needs to be found or re-read on this Mac — e.g. "check the antigravity conversation …",
"what did I discuss in Antigravity about X", a UUID reference with no visible transcript,
or recovery of chat history after an update/crash. Triggers: the words antigravity +
conversation/chat, a bare UUID that resolves nowhere else, or sidebar history loss.

Find and read Google Antigravity (Gemini IDE) conversations on macOS. Antigravity splits
storage into CONTENT (under `~/.gemini/`) and UI INDEX (under `~/Library/Application Support/`).
The `.db` conversations are plaintext SQLite — fully readable without keys.

## Storage Map (verified on macOS, Aug 2026)

```text
~/.gemini/antigravity/                     # legacy app
├── conversations/{UUID}.db                # FULL CHAT — plaintext SQLite (modern)
├── conversations/{UUID}.pb                # older protobuf format (may be encrypted)
├── annotations/{UUID}.pbtxt               # last-viewed timestamps only
└── brain/{UUID}/                          # artifacts + logs for that chat
    └── .system_generated/logs/transcript.jsonl   # CLEAN full transcript (JSONL)
~/.gemini/antigravity-ide/                 # newer split IDE app (same structure)
~/.gemini/antigravity-cli/                 # CLI tool variant (same structure)

~/Library/Application Support/Antigravity/User/globalStorage/state.vscdb
    # ItemTable keys:
    #   antigravityUnifiedStateSync.trajectorySummaries  <- PRIMARY UI INDEX (base64 protobuf)
    #   chat.ChatSessionStore.index                      <- session index (can be emptied by crashes)
# IDE variant app dir: .../Antigravity IDE/...
```

Pitfall: searching `Application Support` alone finds only tab-id references — the actual
chat bodies live under `~/.gemini/`. There are (at least) three content roots:
`antigravity/`, `antigravity-ide/`, and `antigravity-cli/`; a conversation can be in any of
them, so always glob `~/.gemini/antigravity*/conversations/`.

## Operations

All ops are plain shell; run from any cwd.

### 1. Find a conversation file by exact UUID
```bash
ls ~/.gemini/antigravity*/conversations/ | grep -i "<uuid-prefix>"
```

### 2. Search ALL conversations by keyword
Filenames are opaque UUIDs, so sweep the files themselves. SQLite payloads are mostly
uncompressed, so plain `grep -li` is a fast first pass:
```bash
grep -li "monster hunter" ~/.gemini/antigravity*/conversations/*.db
```

### 3. Read one conversation's clean transcript (preferred)
```bash
less "$HOME/.gemini/antigravity/brain/<UUID>/.system_generated/logs/transcript.jsonl"
```
Each line is JSON: `{"step_index","type","created_at","content",...}` with types
`USER_INPUT` / `GENERIC` (tool+model output) / `PLANNER_RESPONSE` / `SYSTEM_MESSAGE`.
Extract just the user prompts:
```bash
python3 -c "
import json,sys
for l in open(sys.argv[1], errors='ignore'):
    j=json.loads(l)
    if j.get('type')=='USER_INPUT':
        c=j['content'].split('<ADDITIONAL_METADATA>')[0]
        print(j.get('step_index'), j.get('created_at'), c.replace('<USER_REQUEST>','').replace('</USER_REQUEST>','').strip()[:200])
" "$HOME/.gemini/antigravity/brain/<UUID>/.system_generated/logs/transcript.jsonl"
```

### 4. Read the raw `.db` when no brain dir exists
```bash
sqlite3 -readonly "$HOME/.gemini/antigravity/conversations/<UUID>.db" \
  "SELECT idx, step_type, length(step_payload) FROM steps ORDER BY idx LIMIT 20;"
# Payloads are protobuf-framed but contain long readable ASCII runs; pull them with:
python3 -c "
import sqlite3,re,sys
con=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True)
for idx,st,p in con.execute('SELECT idx,step_type,step_payload FROM steps ORDER BY idx'):
    if isinstance(p,(bytes,bytearray)):
        for m in re.findall(rb'[\x20-\x7e]{60,}', p):
            print(idx, st, m.decode()[:200])
" "$HOME/.gemini/antigravity/conversations/<UUID>.db"
```
Schema reference: tables `steps` (idx, step_type, status, step_payload, step_format),
`gen_metadata`, `executor_metadata`, `trajectory_meta`, `trajectory_metadata_blob`.

### 5. Cross-reference the sidebar index (find chats invisible in the UI)
```bash
cp "$HOME/Library/Application Support/Antigravity/User/globalStorage/state.vscdb" /tmp/ag.vscdb
sqlite3 /tmp/ag.vscdb "SELECT key, length(value) FROM ItemTable WHERE key LIKE '%trajectory%' OR key LIKE '%Chat%';"
# Diff the UUIDs in trajectorySummaries against `ls ~/.gemini/antigravity*/conversations/`
# Files on disk but absent from the index = chats the UI hides (data still intact).
```

## Decision Guide

| Goal | Use |
| --- | --- |
| Have the UUID | op 3 (brain transcript), else op 4 (.db) |
| Only remember a topic | op 2 grep sweep, then op 3 |
| Chat invisible in sidebar | op 5 cross-reference; data is usually intact on disk |
| Need machine-readable export | transcript.jsonl is already JSONL |

## Safety

Low risk when read-only: always use `sqlite3 -readonly` or `mode=ro` URIs, and copy
`state.vscdb` before querying it (live DB may be locked/WAL). Never write into
`~/.gemini/` while Antigravity is running.

## Integration

Complements `ciel/research` (web) with local-conversation recall; pairs with
`session_search` for Hermes-side history. Discovered while tracing BioGenesis-X
conversation `8dd0fb50-dfc8-4a87-924e-e1056a200de9`.