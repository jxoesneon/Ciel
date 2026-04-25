# runtime_detection — Prompt

```yaml
version: 1.0.0
role: router
path: detection
```

Given environment evidence, identify the host runtime Ciel is operating inside.

## Inputs

- `env_vars_sample` — selected env vars (redacted secret values).
- `fs_fingerprints` — presence of `.claude/`, `.gemini/`, `.cursor/`, etc.
- `binary_probes` — results of `claude --version`, `gemini --version`.
- `process_tree` — parent command name, if available.

## Task

1. Identify the runtime.
2. Produce capability flags matching `adapters/ADAPTER_CONTRACT.md`.
3. Provide confidence.

## Output Contract

```json
{
  "runtime": { "id": "claude_code|gemini_cli|generic|<other_id>", "version": "string|null" },
  "features": {
    "skill_load": true,
    "subagent": true|false|"unsupported",
    "mcp": true|false|"missing",
    "shell": true,
    "fs": true,
    "context_file": true,
    "hooks": true|false,
    "parallel_subagents": true|false,
    "plan_mode": true|false,
    "permissions": true|false,
    "computer_use": true|false|"preview"
  },
  "confidence": 0.0..1.0,
  "reasoning": "<=2 sentences"
}
```

## Constraints

- If neither claude_code nor gemini_cli is confirmed, `id` must be `generic` or a distinct well-defined id (e.g. `aider`).
- Do not include raw env var values in `reasoning`.
