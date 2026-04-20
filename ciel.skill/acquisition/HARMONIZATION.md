# HARMONIZATION

Adapt acquired artifacts to Ciel's repository conventions before Council.

## What Gets Harmonized

| Dimension | Action |
| --- | --- |
| Frontmatter | Ensure `name`, `version`, `description`, `triggers`, `tags`, `runtime_compatibility` |
| Naming | Path converted to `<domain>/SKILL.md` (or `<domain>/<sub>/SKILL.md`) |
| Tags | Mapped into Ciel's taxonomy; unknown tags flagged |
| I/O contract | Expressed as `io_contract` per `registry/SCHEMA.md` |
| Prose style | Short, direct, Ciel's tone (see `core/IDENTITY.md` persona) |
| Dependencies | Declared in `dependencies` — none implicit |
| Source attribution | `source.origin` populated with URLs + fetch hashes |
| License | Added as `source.license` using SPDX id |

## Prompt

`prompts/acquisition/harmonization.md`. Produces a diff against the raw artifact, reviewed by Ciel before Council.

## Rewrite Boundaries

Harmonization **does not**:

- change the underlying behaviour,
- remove attribution to original authors,
- alter license strings.

It changes **presentation**.

## Output

Harmonized `.skill` bundle + a harmonization report attached to the Council run:

```yaml
harmonization_report:
  diffs:
    - path: SKILL.md
      change: "renamed + re-frontmatter"
    - path: README.md
      change: "merged into SKILL.md body"
  unknown_tags: ["obscure-tag"]
  warnings: []
```

## Failure

If harmonization cannot produce a valid shape (e.g. artifact contradicts itself, has no invokable contract), the acquisition fails pre-Council. Logged and source trust reduced.
