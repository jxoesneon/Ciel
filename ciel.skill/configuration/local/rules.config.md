# rules.config — Local Project Rules

Codified rules specific to this project. Extracted from CLAUDE.md / GEMINI.md + detected conventions + user additions.

```yaml
# <anchor:start>
rules:
  forbidden_ops: []
    # examples:
    # - "git push --force"
    # - "Bash(rm:*) outside ./tmp"
  required_patterns: []
    # - "all functions must have type hints"
    # - "all new endpoints must have at least one test"
  style:
    formatter: null
    linter: null
    eol: lf
  testing:
    require_tests_for: [new_endpoints, bug_fixes]
    min_coverage_pct: null
  security:
    never_commit: [".env", "secrets/"]
    pii_handling: null                # null | mask | redact | block
  documentation:
    require_docstrings: false
    require_changelog_entry: false
# <anchor:end>
```

## Notes

- Rules here **override** global defaults for this project.
- They do **not** override Constitutional invariants or Safety veto conditions.
- Coherence member validates rule consistency at Council opportunities.

## Adding Rules

- Edit this file directly, OR
- `/ciel rule add "<natural description>"` — Ciel converts to structured form, proposes via Council (standard).

## Provenance

Each rule may carry a comment about source (`# detected from existing .prettierrc`, `# user-added 2026-01-15`).
