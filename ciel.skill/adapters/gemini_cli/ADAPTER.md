# ADAPTER — Gemini CLI

Full capability adapter for Gemini CLI.

## Capability Flags

```yaml
runtime: gemini-cli
floor: { skills: true, subagents: true, mcp: true, shell: true, fs: true, context: true }
enhanced:
  hooks: true
  parallel_subagents: true       # native, with recursion guard
  plan_mode: true                # read-only pre-check
  sandboxing: true               # macOS Seatbelt / Docker / Podman
  checkpointing: true
  a2a_remote_subagents: true     # Agent-to-Agent protocol
  multimodal: true               # Imagen / Veo / Lyria
  token_cache: true
  model_routing: true            # automatic fallback
  native_context: 1000000        # 1M on Gemini 3
```

## File Layout Expected

- `.gemini/agents/` — subagent markdowns (+ `@syntax` references)
- `.gemini/extensions/` — extension + playbook bundles
- `~/.gemini/settings.json` — MCP + prefs
- `GEMINI.md` — project context
- `~/.gemini/skills/` — installed skills

## Installation Footprint

At init, Ciel:

1. Drops slash-equivalent commands into `.gemini/commands/` (`ciel`, `ciel-init`, etc.).
2. Registers hook scripts under `.gemini/hooks/`.
3. Ensures `GEMINI.md` exists with a Ciel-anchored block.
4. Installs council members as subagent files.

## Route Map

| Ciel route | Gemini CLI mechanism |
| --- | --- |
| Skill activation | `activate_skill` tool + SKILL.md |
| Subagent delegation | `.gemini/agents/*.md` + `@` syntax |
| Parallel dispatch | native parallel subagents |
| Pre-flight gate | Plan mode (read-only pre-check) |
| Post-execution scoring | hooks + telemetry diff |
| MCP | `~/.gemini/settings.json` servers |
| Context injection | `GEMINI.md` + extensions playbooks |
| Long task | checkpointing + A2A remote subagents |
| Media generation | Imagen / Veo / Lyria extensions |
| Shell isolation | Seatbelt / Docker / Podman |
| Headless script | `gemini --output-format json` |

See per-file specs: `HOOKS.md`, `SUBAGENTS.md`, `EXTENSIONS.md`, `PLAN_MODE.md`, `CHECKPOINTING.md`, `A2A.md`, `MULTIMODAL.md`, `TOKEN_CACHE.md`, `MODEL_ROUTING.md`, `INTERACTIVE_SHELL.md`, `COUNCIL_INVOCATION.md`, `CONTEXT_FILES.md`.

## Council Invocation Strategy

Gemini CLI enforces subagent **non-recursion**. Council of Five runs as five parallel **top-level** subagents reporting to Ciel (the chairman lives in the root session). See `COUNCIL_INVOCATION.md`.
