---
name: web_fetch
version: 1.0.0
description: URL fetching with HTML/JSON/markdown extraction, pagination, retry.
triggers: [fetch, download, curl, get url, visit]
tags: [network, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }

dependencies: { skills: [], mcp: [], system: [] }
---
# web_fetch

Fetch a URL.

## Operations

- `fetch.get(url, headers?)` — returns body + metadata + fetch_hash.
- `fetch.json(url)` — parsed JSON.
- `fetch.md(url)` — HTML→markdown extraction.
- `fetch.paginate(url, pattern)` — follow next links.

## I/O Contract

```yaml
io_contract:
  input: { url, "headers?", "timeout_s?" }
  output: { status, body, headers, fetch_hash, final_url }
  idempotent: true
  side_effects: [network]
```

## Safety

- Robots.txt respected.
- Redirects capped at 10.
- Body size capped by `acquisition.config.total_wall_budget_s` proxy (configurable).
- All fetches logged with URL hashed if `redact_urls: true`.
- Bodies stored in MemPalace by `fetch_hash` for audit.
