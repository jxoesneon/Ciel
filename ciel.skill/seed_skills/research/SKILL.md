---
name: research
version: 1.0.0
description: Deep research protocol — multi-source synthesis, confidence assessment, citations.
triggers: [research, deep research, investigate, look up]
tags: [network, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [web_search/SKILL.md, web_fetch/SKILL.md, context_summarizer/SKILL.md] }
---
# research

Multi-source synthesis with citations.

## Operations

- `research.deep(question, budget_tokens?)` — iterative search + fetch + synthesize.
- `research.compare(a, b)` — side-by-side analysis.
- `research.verify(claim, sources?)` — evidence-based check.
- `research.outline(topic)` — structured outline to drive follow-up.

## I/O Contract

```yaml
io_contract:
  input: { op, question, "budget_tokens?", "since?" }
  output: { summary, citations: [ { url, excerpt, confidence } ], confidence }
  idempotent: true (approximate)
  side_effects: [network]
```

## Principles

- Multi-source required; never trust a single page.
- Official docs > community > blog > forum.
- Citations stored in MemPalace for audit; `fetch_hash` preserved.
- Returns an honest confidence estimate; below threshold triggers user escalation.
