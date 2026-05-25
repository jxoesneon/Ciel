# SUBAGENTS — Devin for Terminal

Devin for Terminal supports both built-in and custom subagent profiles. Ciel leverages these for Council of Five deliberation, parallel research, and specialized task delegation.

## Built-In Profiles

| Profile | Description | Tool Access |
|---------|-------------|-------------|
| `subagent_explore` | Read-only codebase exploration | Read-only tools only |
| `subagent_general` | General-purpose tasks | Full tools (foreground) or pre-approved only (background) |

## Custom Subagent Profiles

Define custom profiles in `.devin/agents/<name>/AGENT.md`:

```markdown
---
name: ciel-chairman
description: Ciel Council of Five — Chairman orchestrates deliberation
model: sonnet
allowed-tools:
  - read
  - grep
  - glob
  - exec
permissions:
  allow:
    - Exec(git)
    - Read(**)
---

You are the Chairman of Ciel's Council of Five. Your role is to orchestrate
multi-member deliberation on skill acquisition, self-modification, and
high-risk operations.

Process:
1. Present the proposal to all five members
2. Collect scores from each member
3. Facilitate Stage 2 cross-review
4. Determine majority (>= 3/5) and Safety veto status
5. Report the final decision with rationale
```

## Ciel's Council of Five as Subagents

```
.devin/agents/
├── ciel-chairman/
│   └── AGENT.md    # Orchestrates deliberation
├── ciel-safety/
│   └── AGENT.md    # Veto authority — blocks unsafe proposals
├── ciel-efficiency/
│   └── AGENT.md    # Flags bloat, duplication, perf issues
├── ciel-coherence/
│   └── AGENT.md    # Checks consistency with existing skills/registry
└── ciel-evidence/
│   └── AGENT.md    # Validates research and citations
```

## Invocation

From the main agent, invoke a Council subagent:

```
Run a subagent with the ciel-chairman profile to deliberate on
this skill acquisition proposal: <proposal details>
```

Or use the `agent` field in a skill's frontmatter:

```yaml
---
name: ciel-council
description: Invoke the Council of Five for deliberation
agent: ciel-chairman
---

Deliberate on the following proposal and return a structured decision.
```

## Background vs Foreground

- **Foreground**: Parent waits; you can approve/deny tool calls as they arise. Use for complex deliberations where the Chairman may need to read files.
- **Background**: Parent continues working. Use for independent research tasks (e.g., `subagent_explore` profiling a new skill source).

## Nesting Limitations

Subagents cannot spawn their own subagents. Only the root agent can spawn subagents. This means Ciel's Council cannot recursively delegate — the Chairman must handle all member interactions inline or via structured prompt decomposition.

## Tool Permissions in Subagents

- Foreground subagents: behave like main agent (prompt for approval)
- Background subagents: inherit pre-approved permissions; unapproved tools are auto-denied

## Best Practices

1. Use `subagent_explore` for registry research and skill gap detection
2. Use `subagent_general` for implementation and testing tasks
3. Use custom `ciel-*` profiles for deliberation and governance
4. Keep subagent prompts focused — one clear task per subagent
5. Set `timeout` on hook commands to prevent hanging the parent
