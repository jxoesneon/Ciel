---
name: web_search
version: 1.0.0
description: Web search with query construction, result parsing, source ranking.
triggers: [search, web search, google, find online]
tags: [network, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [web_fetch/SKILL.md] }
---
# web_search

Query a search engine and rank results.

## Operations

- `search.query(terms, filters)` — returns ranked list of `{title, url, snippet, score}`.
- `search.news(terms, since)` — news filter.
- `search.code(terms, lang?)` — code-focused (GitHub, Sourcegraph).

## I/O Contract

```yaml
io_contract:
  input: { terms: string, "filters?": map, "top_k?": int }
  output: { results: [ { title, url, snippet, score } ] }
  idempotent: true
  side_effects: [network]
```

## Providers

Host-native when possible (Claude web search, Gemini grounded). Falls back to configured `search.provider` with API key from `secrets_manager`.

## Safety

- Queries logged.
- URL fetches defer to `web_fetch/SKILL.md`.
- Safe-search defaults on.
