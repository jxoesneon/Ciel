---
name: json_yaml_toml_parser
version: 1.0.0
description: JSON / YAML / TOML — read, write, validate, transform, schema-check.
triggers: [json, yaml, toml, parse, config]
tags: [code, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [filesystem/SKILL.md] }
---
# json_yaml_toml_parser

Structured config parsers.

## Operations

- `cfg.read(path)` — detects format by extension.
- `cfg.write(path, obj, format?)` — preserves comments for YAML/TOML where possible.
- `cfg.validate(obj, schema)` — JSON Schema.
- `cfg.transform(path, patch)` — path-based updates; preserves shape.
- `cfg.merge(a, b, strategy=deep|shallow)`.

## I/O Contract

```yaml
io_contract:
  input: { op, path_or_obj, args }
  output: { obj_or_ok }
  idempotent: for_same_inputs
  side_effects: ["fs?"]
```

## Safety

- Schema validation before write when a schema is available.
- Refuses to write if the result would be malformed.
