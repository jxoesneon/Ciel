# REASONING_PATH

LLM-driven composition for novel or ambiguous requests. Invoked when the fast path misses.

## Goal

Decide whether the registry can satisfy the request by composing existing skills, and if so, produce an ordered execution plan.

## Inputs

- Request envelope (see `router/ROUTER.md`).
- Registry metadata slice (not full skills) — all skill `SKILL.md` frontmatter + one-paragraph description.
- Recent execution trace slice from MemPalace for continuity.

## Prompt

Uses `prompts/router/reasoning_path.md`. The prompt asks the host LLM to:

1. Identify the atomic sub-tasks the request decomposes into.
2. For each sub-task, match the best registered skill OR mark as **gap**.
3. Produce a plan: ordered list of `(skill_id, input_shape)`.
4. Return a self-reported confidence 0..1.

## Output Contract

```json
{
  "plan": [ { "skill": "git/SKILL.md", "input": {...} }, ... ],
  "gaps": [ { "subtask": "...", "reason": "..." } ],
  "confidence": 0.72
}
```

## Confidence Floor

`router.config.reasoning_floor` (default `0.70`).

- ≥ floor, no gaps → execute plan.
- ≥ floor, with gaps → partial-execute known steps, route gaps to `ACQUISITION_PATH.md`.
- < floor → route whole request to `ACQUISITION_PATH.md`.

## Composition

Multi-step plans are executed sequentially by default. Steps are independent only if the plan explicitly marks them so. Parallel execution is delegated to the host runtime adapter (Gemini CLI parallel subagents / Claude Code multi-tool).

## Failure Handling

- If any sub-step errors, the plan halts, state is captured, and failure flows into `self_improvement/TRIGGERS.md`.
- If an errored step has a declared idempotency flag, retry once with exponential backoff before halting.

## Cost Awareness

Reasoning path consumes host LLM tokens. Each invocation is budgeted by `configuration/global/router.config.md`. Exceeding budget escalates to user with an explanation.

## Telemetry

```json
{ "path": "reasoning", "confidence": 0.78, "steps": 3, "gaps": 0, "ms": 940 }
```
