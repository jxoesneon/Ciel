# RESEARCH_PROTOCOL — Generic

When Ciel encounters an unknown runtime, she kicks off a research cycle to produce a dedicated adapter.

## Pipeline

1. **Identify the runtime**
    - Inspect environment variables, binary names, filesystem layout.
    - Check known tool directories (`/usr/local/bin`, `~/.local/bin`).
    - Read any `README` in the current session's working dir for hints.

2. **Research the runtime**
    - Use `seed_skills/research/SKILL.md` with query template:
      `"<runtime> CLI agent skill format hooks MCP subagents"`.
    - Gather docs, GitHub readmes, release notes.
    - Deduce capability matrix aligned with `adapters/ADAPTER_CONTRACT.md`.

3. **Probe**
    - Run `adapters/generic/CAPABILITY_PROBE.md` to confirm the documented capabilities.
    - Mark mismatches and record them.

4. **Draft adapter**
    - `seed_skills/runtime_adapter_builder/SKILL.md` instantiates `templates/adapter.template.md`.
    - Fill in capability flags, route table, installation footprint.

5. **Sandbox test**
    - Minimal test: load `ciel.skill` in the new runtime, execute three canned requests.
    - Collect telemetry.

6. **Council**
    - Full Council of Five reviews the drafted adapter under `council/invocation_scopes/SKILL_INTEGRATION.md` (adapter = specialized skill).

7. **Install**
    - Council pass → write to `adapters/<runtime_id>/`, commit, update `MANIFEST.md`, increment minor version.

## Output

A new directory `adapters/<runtime_id>/` with full file set analogous to `claude_code/` or `gemini_cli/`.

## Failure

If research cannot confirm the floor, the runtime is marked `incompatible` in the registry. Ciel continues with Generic minimal mode for that runtime and revisits on major version bumps of the host tool.
