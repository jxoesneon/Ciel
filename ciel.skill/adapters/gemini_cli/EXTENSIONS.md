# EXTENSIONS — Gemini CLI

Extensions bundle MCP servers, context files, and instructions under a single installable unit with an executable *playbook* — an intelligence layer above raw MCP connectivity.

## Why Ciel Uses Extensions

- Whole-capability packaging: a single extension gives Ciel multiple tools + a recipe for using them.
- Environment-aware execution: playbooks evaluate local files / git status before choosing a tool.
- Version-pinnable: extensions have versions; Ciel can roll back.

## Ciel's Own Extension

At init, Ciel may optionally install herself as a Gemini extension: `ciel.extension` — a small wrapper that simply activates the `ciel.skill` and announces slash commands.

## Discovery

Gemini CLI extension registry is queried by `acquisition/TIER_2_MCP.md` for Tier 2 matches. Extensions matching a capability gap are evaluated by the Council of Five before installation.

## Playbook Shape

```yaml
extension:
  name: example-extension
  mcp_servers: [...]
  context_files:

    - path: GEMINI.md

      inject: extension-guide.md
  playbook: |
    When <trigger>:

      1. run <tool>
      2. if <condition>: <tool2>
      3. else: <tool3>

```

Ciel treats playbooks as read-only external skills. She does not edit an installed extension's files — if she needs to adapt behaviour, she composes a new wrapper skill under her own registry.

## Safety

Extensions execute in the Gemini CLI sandbox; Ciel enforces that new extensions start with the most restrictive sandbox profile available (Seatbelt or Docker) and are only loosened after demonstrated need + Council vote.
