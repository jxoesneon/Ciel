# A2A — Agent-to-Agent Remote Subagents

Gemini CLI supports remote subagents via the Agent-to-Agent protocol. A remote subagent is described by an `agent_card_url` which declares its capabilities, endpoints, and auth model.

## When Ciel Uses A2A

- A capability exists only as a remote service (e.g. a hosted specialized planner).
- The user has opted in to external agents via `configuration/global/adapters.config.md`.
- The task is isolated and its inputs/outputs fit A2A's message contract.

## Agent Card Verification

Before invoking a remote subagent, Ciel:

1. Fetches the agent card.
2. Verifies TLS + signature (if provided).
3. Presents the capability description and requested scopes to the user for first-use approval.
4. Stores the approval decision in MemPalace (trust).

## Invocation

```yaml
a2a:
  agent_card_url: "https://example.com/.well-known/agent.json"
  scopes: [read-only, analyze]
  timeout_s: 120
  context_fields: [task, constraints]
```

## Safety

Remote subagents cannot access Ciel's memory partition or the project filesystem directly. Only serialized inputs and outputs cross the boundary. Safety member enforces this at invocation time.

## Logging

Every A2A call is recorded in `activity.log` with:

- remote agent id,
- input/output byte counts,
- latency,
- trust delta (if success / failure adjusts trust).

## Fallback

If A2A is unreachable, Ciel tries a local equivalent skill or escalates. A2A is never a hard dependency.
