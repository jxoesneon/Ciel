---
name: token-budget-advisor
version: 1.0.0
format: skill/1.0
description: A user-facing depth control that offers choices regarding response length and token expenditure before answering.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(token budget|depth|length|detailed|brief|tldr)"

    confidence: 0.8

  - pattern: "respond at (25|50|75|100)%"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Token Budget Advisor (Depth Control)

This skill allows the user to explicitly control the "Resolution" of CIEL's responses. It intercepts the response flow to offer a choice between Essential, Moderate, Detailed, and Exhaustive depth levels.

## The Depth Matrix

| Level | Target | Includes |
| :--- | :--- | :--- |
| **Essential (25%)** | 2-4 sentences | Direct answer/conclusion only. No preamble. |
| **Moderate (50%)** | 1-3 paragraphs | Answer + key context + one example. |
| **Detailed (75%)** | Structured | Full answer + alternatives + edge cases. |
| **Exhaustive (100%)** | Unrestricted | Deep analysis + all perspectives + code. |

## Operational Heuristics

1. **Estimate**: Use `words * 1.3` (prose) or `chars / 4` (code) to estimate prompt tokens.
2. **Present**: Offer the 4 levels BEFORE answering if the user mentions budget, length, or depth.
3. **Execute**: Adhere strictly to the chosen level's target length and inclusion rules.
4. **Persist**: Maintain the chosen level for the remainder of the session unless overridden.

## When to Skip

Do NOT trigger the advisor if:

- The user has already selected a level this session.
- The answer is naturally a single line (e.g., "Yes" or a path).
- The term "token" refers to auth/session tokens rather than context.

## Anti-Patterns

- **Level Drift**: Offering a summary then expanding into an exhaustive answer anyway.
- **Token Hallucination**: Claiming exact token counts without stating they are heuristic estimates (±15%).
