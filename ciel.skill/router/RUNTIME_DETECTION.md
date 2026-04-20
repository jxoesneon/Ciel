# RUNTIME_DETECTION

Fingerprint the host runtime on every invocation. Cache the result per session.

## Probe Order

1. **Environment variables** (cheapest)
    - `CLAUDE_CODE_VERSION` / `CLAUDECODE=1` → **claude-code**
    - `GEMINI_CLI_VERSION` / `GEMINI_API_KEY` + CLI binary present → **gemini-cli**
    - `CURSOR_*` / `WINDSURF_*` / `AIDER_*` → flagged as other known runtimes
2. **Filesystem fingerprints**
    - `.claude/` directory, `~/.claude/settings.json` → claude-code
    - `.gemini/` directory, `~/.gemini/settings.json` → gemini-cli
3. **Binary probes** (if `seed_skills/shell/SKILL.md` available)
    - `claude --version` success → claude-code
    - `gemini --version` success → gemini-cli
4. **Capability probe** (generic fallback)
    - Defer to `adapters/generic/CAPABILITY_PROBE.md`.

## Output

```yaml
runtime:
  id: claude-code | gemini-cli | generic
  version: "..."
  features:
    hooks: true
    subagents: true
    mcp: true
    parallel_subagents: false | true
    computer_use: false | preview | true
    plan_mode: false | true
```

## Adapter Load

Based on `runtime.id`, Ciel loads `adapters/<id>/ADAPTER.md`. If `id == generic`, loads `adapters/generic/ADAPTER.md` and initiates `adapters/generic/RESEARCH_PROTOCOL.md`.

## Cache

Cached in-session only — runtime is considered invariant within a session. On session start, re-run. Store last-seen runtimes in MemPalace for `reasoning_path` priors.

## Ambiguity

If multiple runtimes fingerprint simultaneously (uncommon), prefer the environment variable that is set in the current process (pre-empted by the one that invoked Ciel). Log both.

## Prompt

`prompts/router/runtime_detection.md` is used when neither environment nor filesystem probes disambiguate.

## Unknown Runtime

Unknown → `generic` adapter. Ciel will also enqueue a background self-improvement task to build a dedicated adapter: see `seed_skills/runtime_adapter_builder/SKILL.md`.
