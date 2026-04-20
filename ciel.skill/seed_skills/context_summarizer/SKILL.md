---
name: context_summarizer
version: 1.0.0
description: Context compression for memory — relevance scoring, AAAK formatting.
triggers: [summarize, compress context, aaak]
tags: [memory, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [mempalace_manager/SKILL.md] }
---

# context_summarizer

Compress long traces / conversations into recoverable AAAK blobs.

## Operations

- `ctx.summarize(text, target_tokens, anchor_hints?)` — produce AAAK blob.
- `ctx.expand(blob)` — reconstruct best-fidelity form.
- `ctx.rank(chunks, query)` — relevance scores for eviction.
- `ctx.evict(segments, budget)` — compose an eviction plan.

## I/O Contract

```yaml
io_contract:
  input: { op, args }
  output: { result }
  idempotent: deterministic_with_seed
  side_effects: []
```

## AAAK

Adaptive Anchored Attention Keys — lossy-compressed format with recovery anchors. Written to MemPalace as native `.aaak` blobs.
