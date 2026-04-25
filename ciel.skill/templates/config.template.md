# Template — new config file

New `configuration/global/<name>.config.md` or local equivalent.

```markdown

# {{name}}.config — {{scope}}

```yaml
<anchor:start>
{{field}}:
  {{subfield}}: {{default}}
<anchor:end>
```

## Notes

- Purpose: {{what this config controls}}.
- Defaults: {{default values + rationale}}.
- Constitutional floors: {{if any}}.
- Self-tuning range: {{±X or "user-only"}}.

## Related

- `configuration/SCHEMA.md`
- `configuration/DEFAULTS.md`
- `configuration/TUNING.md`

```text
