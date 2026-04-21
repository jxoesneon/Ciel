# SUBAGENTS — Windsurf Adapter

Windsurf Cascade does not support native subagent spawning. Ciel emulates subagents via inline persona switching.

## Inline Subagent Pattern

Instead of spawning agents, Ciel structures prompts with clear persona headers:

```markdown
# Subagent: {name}

## Persona
{persona from .windsurf/skills/ciel/agents/{name}.md}

## Input
{task input}

## Your Task
Execute as this persona. Return only the result, no meta-commentary.

## Result
(You respond here as the persona)
```

## Sequential Execution

Multi-step workflows run as sequential inline personas:

1. Router persona → determines path
2. Researcher persona → gathers info
3. Executor persona → performs action
4. Validator persona → checks result

## Limitations

- No parallel execution (tasks run serially)
- Higher context usage (personas loaded inline)
- No true isolation (all in same context window)

## Mitigations

1. **Compact personas**: Keep agent definitions under 200 tokens
2. **Summarize between steps**: Pass only essential state forward
3. **Early termination**: Stop chain if intermediate result fails
4. **Checkpoint pattern**: Log progress to `.ciel/activity.log` for resume

## Agent Directory

Ciel stores persona files at `.windsurf/skills/ciel/agents/`:

- `router.md` — routing decisions
- `acquirer.md` — skill acquisition
- `composer.md` — skill composition
- `councilor.md` — council deliberation
- `researcher.md` — research tasks

## Migration Path

If Windsurf adds native subagents in future, this adapter upgrades to parallel mode with no skill changes.
