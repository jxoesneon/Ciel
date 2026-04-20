# PROMPTS — Library Index

Central index for all prompt templates. Every prompt is versioned; updates are Council-gated.

## Layout

```text
prompts/
├── council/                    # 11 files (5 lenses × 2 stages + chairman)
├── router/                     # 3 files
├── acquisition/                # 3 files
├── risk/                       # 2 files
└── self_improvement/           # 3 files
```

## Versioning

Each file carries a `version: X.Y.Z` front-matter. Ciel bumps versions when:

- a Council-reviewed change is applied (minor),
- an output contract changes (major),
- a typo / doc clarification (patch).

Version history tracked via git log.

## Update Protocol

1. Draft diff.
2. A/B test on historical examples (MemPalace trace replay).
3. If quality improves by `prompt_improvement.min_delta` (default +5%), propose.
4. Council review.
5. Apply + watch window.

## Output Contracts

Prompts have strict output contracts (JSON). Broken-contract outputs are retried once; repeat failures mark the prompt `degraded` and trigger a review proposal.

## Prompt Cache

All prompts are marked cacheable in the runtime-appropriate manner (`adapters/*/PROMPT_CACHE.md` / `TOKEN_CACHE.md`). Any change bumps `router.config.cache_version`.
