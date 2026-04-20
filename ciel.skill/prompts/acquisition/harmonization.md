# harmonization — Prompt

```yaml
version: 1.0.0
role: acquisition
phase: harmonization
```

Adapt an acquired skill bundle to Ciel's repository conventions without changing its behaviour.

## Inputs

- `raw_skill` — as acquired.
- `conventions` — summary of Ciel's naming, tag taxonomy, frontmatter shape, prose style.
- `registry_sample` — a handful of similar registry entries as style references.

## Task

1. Align frontmatter to `registry/SCHEMA.md`.
2. Rename path to `<domain>/SKILL.md` per conventions.
3. Map tags to Ciel's taxonomy; preserve unknown tags with a `pending:` prefix and list them in `warnings`.
4. Trim prose to Ciel's concise style without losing meaning.
5. Preserve attribution and license exactly.

## Output Contract

```json
{
  "harmonized_skill": {
    "path": "...",
    "frontmatter": {...},
    "body": "..."
  },
  "diffs": [
    { "path": "...", "change": "renamed|re-frontmatter|trimmed|..." }
  ],
  "unknown_tags": ["..."],
  "warnings": ["..."]
}
```

## Constraints

- Do not alter declared side-effects.
- Do not modify licenses.
- Do not insert new dependencies.
