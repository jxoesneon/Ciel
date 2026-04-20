# CAPABILITY_PROBE — Generic

A scripted probe sequence to discover what an unknown runtime can actually do. Used by `adapters/generic/ADAPTER.md` at load and by `RESEARCH_PROTOCOL.md`.

## Probe Sequence

| Probe | Method | Confirms |
| --- | --- | --- |
| skill-load | ask host to load `SKILL.md` then re-read its own system prompt | SKILL.md loading |
| subagent | attempt to spawn a minimal subagent; if unsupported, fail gracefully | subagents |
| mcp | inspect env / settings for MCP endpoint; if present, ping | MCP client |
| shell | run `echo ciel-probe-$RANDOM`, parse output | shell execution |
| fs | write tmp file, read, delete | filesystem I/O |
| context-file | write marker to `CLAUDE.md` / `GEMINI.md` / `AGENT.md` / create if missing | context file |
| hooks | check runtime config for hook registration keys | hooks (enhanced) |
| parallel | attempt two subagent calls in one turn | parallel subagents |
| plan-mode | look for `--plan`, read-only flag | plan mode |
| permissions | look for `allowedTools` / permission config | permissions |
| computer-use | look for desktop/UI primitives | computer use |

## Output

```yaml
probe_results:
  runtime_guess: "<id>"
  floor:
    skill_load: ok
    subagent: ok|unsupported
    mcp: ok|missing|degraded
    shell: ok
    fs: ok
    context_file: ok|synthesized
  enhanced:
    hooks: ok|absent
    parallel_subagents: ok|absent
    plan_mode: ok|absent
    permissions: ok|absent
    computer_use: ok|absent
  confidence: 0.82
```

Serialized to MemPalace partition `ciel/runtimes/<runtime_guess>/probe-<timestamp>`.

## Interpretation

- All six floor probes `ok` → generic operation possible.
- Any floor probe failure → Ciel enters minimal mode + enqueues research + notifies user.
