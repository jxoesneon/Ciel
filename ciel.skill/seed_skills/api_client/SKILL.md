---
name: api_client
version: 1.0.0
description: HTTP REST / GraphQL — request construction, auth, error handling, retry.
triggers: [api, http, rest, graphql, request, endpoint]
tags: [network, scope:both, runtime:any, risk:mid]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [secrets_manager/SKILL.md, web_fetch/SKILL.md] }
---
# api_client

HTTP REST / GraphQL client with auth and retry.

## Operations

- `api.request(method, url, headers?, body?, auth?)`
- `api.graphql(endpoint, query, vars?, auth?)`
- `api.with_retry(call, strategy)` — exponential backoff, idempotency keys.

## Auth

Auth credentials pulled via `secrets_manager/SKILL.md` by name — never inline. Bearer, Basic, OAuth2, API-key header, HMAC.

## I/O Contract

```yaml
io_contract:
  input: { method, url, "headers?", "body?", "auth_ref?" }
  output: { status, body, headers }
  idempotent: depends_on_method
  side_effects: [network]
```

## Safety

- State-mutating methods (POST/PUT/PATCH/DELETE) to external endpoints are at least mid-risk.
- Sending to internal / production endpoints is high/critical.
- Request/response bodies stored hashed, not raw, for audit.
