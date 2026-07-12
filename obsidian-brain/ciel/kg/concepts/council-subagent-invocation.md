---
title: Council Subagent Invocation
type: concept
aliases: [council subagents]
tags: [concept, ciel, council, self-improvement]
created: 2026-07-08
status: active
---

# Council Subagent Invocation

A rule for how Ciel must run the Council of Five.

## Rule

When the user asks for the Council of Five and the runtime supports subagents, Ciel must run each member (Coherence, Capability, Safety, Efficiency, Evolution) as a separate subagent with isolated context. Each subagent evaluates the case through only its own lens and returns a structured score.

## Fallback

Ciel falls back to monolithic synthesis by the Chairman only when no authenticated subagent runtime is available in the current session. This fallback must be recorded in the audit.

## Implementation

- Use the `claude` CLI with `claude -p <prompt> --no-persistence` for each member.
- Prompt includes the member persona from `ciel.skill/council/members/<MEMBER>.md`, the case text, and a strict JSON output contract.
- Collect all five Stage 1 results, then synthesize Stage 3 using the weights in `ciel.skill/council/rubrics/SCORING.md`.
- Preserve the reusable runner at `scripts/council/run-subagent-audit.mjs`.

## History

- 2026-07-08 — User instructed Ciel to apply this rule. First attempt on the Obsidian brain audit failed because the `claude` CLI was not logged in. Fallback to monolithic synthesis was recorded in `ciel/kg/decisions/obsidian-brain-migration-audit`.

## Related

- [[ciel/kg/decisions/obsidian-brain-migration-audit]]
- [[_CLAUDE.md]]
- `ciel.skill/council/COUNCIL.md` (source of truth in the Ciel repository)
