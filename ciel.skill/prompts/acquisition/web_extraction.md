# web_extraction — Prompt

```yaml
version: 1.0.0
role: acquisition
tier: 3
```

Given a set of web documents relevant to a capability gap, extract and synthesize a candidate `.skill` bundle.

## Inputs

- `gap_description` — from `gap_detection.md`.
- `sources` — list of `{url, fetch_hash, excerpt}` for each fetched document.
- `target_id` — proposed canonical id.
- `tags`, `triggers`, `io_contract_shape` — hints from gap detection.

## Task

1. Extract the minimal set of commands / API calls / patterns from sources that fulfill the gap.
2. Compose a cohesive `SKILL.md` body.
3. Note declared side-effects honestly.
4. Cite every source used (URL + fetch_hash).

## Output Contract

```json
{
  "skill": {
    "path": "<id>/SKILL.md",
    "frontmatter": {
      "name": "...",
      "version": "0.1.0",
      "description": "...",
      "triggers": [...],
      "tags": [...],
      "runtime_compatibility": {...},
      "license": "<SPDX>",
      "source": { "tier": 3, "origin": [{"url":"...","hash":"..."}] }
    },
    "body": "markdown body"
  },
  "side_effects_declared": ["shell"|"fs"|"network"|...],
  "notes": "..."
}
```

## Constraints

- Do not fabricate URLs or hashes.
- Do not include content from sources without attribution.
- Do not emit executable code without a clear behaviour description.
- If sources disagree, prefer official docs over community blogs.
