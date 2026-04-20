# gap_detection — Prompt

```yaml
version: 1.0.0
role: router
path: acquisition
```

Ciel has detected a gap. Given the failed fast and reasoning paths, describe the gap precisely enough to drive tiered acquisition.

## Inputs

- `request`, `project_ctx`, `runtime`, `risk_class`.
- `fast_path_diag` — why it missed (top candidates + scores).
- `reasoning_path_diag` — why it missed (gaps list from the planner).

## Task

1. Synthesize a crisp one-paragraph description of the missing capability.
2. Propose canonical triggers and tags.
3. Propose I/O contract shape.
4. Recommend tier order (default `[1,2,3]`; may rearrange with rationale — e.g. the gap is obviously MCP-shaped).

## Output Contract

```json
{
  "gap_description": "string",
  "canonical_id_suggestion": "<domain>/<sub>/SKILL.md",
  "triggers": ["..."],
  "tags": ["..."],
  "io_contract_shape": { "input": {...}, "output": {...} },
  "tier_order": [1, 2, 3],
  "confidence": 0.0..1.0
}
```

## Constraints

- Be concrete; avoid generic descriptions like "does X".
- Tags should come from the established taxonomy.
