# Template — runtime-agnostic subagent

Renders into `.claude/agents/<name>.md` or `.gemini/agents/<name>.md` with runtime-appropriate frontmatter shape.

## Frontmatter (canonical)

```yaml
---
name: {{agent_id}}
description: {{one-line purpose}}
tools: {{[Read, Grep, Bash, WebFetch, ...] }}
model: {{runtime-specific id | auto}}
---
```

## Body

```markdown
You are {{persona}}.

## Context
{{bundle: task description, scope preamble, rubric, prompts}}

## Task
{{numbered steps}}

## Output Contract
{{strict JSON shape the caller will parse}}

## Constraints
{{list}}
```

## Adaption

- Claude Code: `tools` uses Claude tool names.
- Gemini CLI: `tools` uses Gemini tool names; `model` may specify Gemini model.
- Generic: subagent inlined into parent context if runtime lacks subagent primitive.
