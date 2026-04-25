# project.config — Local Project

```yaml

# <anchor:start>

project:
  # Populated by init/CONTEXT_DETECTION.md. Do not hand-edit unless correcting auto-detection.
  id: null
  root: null
  language: null
  frameworks: []
  build_tools: []
  test_tools: []
  formatter: null
  linter: null
  ci: []
  license: null
  monorepo: false
  runtime_hint: null
  detected_at: null

# <anchor:end>

```

## Notes

- This file captures the **auto-detected** project fingerprint.
- Corrections (wrong language, missed framework) can be hand-edited; Ciel respects them on next init integrity pass.
- To re-run detection, `/ciel-init --redetect`.

## Related

- `configuration/local/rules.config.md` — codified project rules.
- `configuration/local/overrides.config.md` — config overrides.
- `configuration/local/escalation.config.md` — escalation threshold.
