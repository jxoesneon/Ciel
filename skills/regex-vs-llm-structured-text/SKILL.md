---
name: regex-vs-llm-structured-text
version: 1.0.0
format: skill/1.0
description: A cost-optimization framework for choosing the right tool (Regex vs. LLM) for structured text parsing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "parse (quiz|form|invoice|structured text)"
    confidence: 0.9
  - pattern: "(regex vs llm|regex or llm)"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Regex vs. LLM (Parsing Strategy)

This skill provides a decision framework for structured text extraction, prioritizing deterministic speed (Regex) for the common case and reserving LLM reasoning for the long tail.

## The Hybrid Pipeline Pattern
1. **Regex Parser (95%)**: Attempt to extract structure using deterministic regular expressions. Optimized for speed and cost.
2. **Confidence Scorer**: Programmatically flag items that deviate from expected patterns (e.g., missing fields, short text).
3. **LLM Validator (5%)**: Dispatch low-confidence items to a cheap LLM (e.g., Haiku) for intelligent extraction/fixing.

## Decision Matrix
- **Use Regex IF**: The format is consistent (>90% repeating), and you need deterministic speed.
- **Use LLM IF**: The text is free-form, highly variable, or the regex parser returns low confidence (<0.95).

## Efficiency Metrics (Target)
- **Cost Savings**: >90% reduction compared to an all-LLM pipeline.
- **Accuracy**: >99% combined accuracy (Regex speed + LLM reasoning).
- **Latency**: Near-instant processing for the majority of inputs.

## Implementation Standards
- **Chunking**: For large documents, chunk the input to prevent regex timeouts.
- **Test-Driven Development**: Write tests for known patterns first, then add LLM fallbacks for the edge cases.
- **Statelessness**: Parsers should be pure functions that do not mutate input state.

## Anti-Patterns
- **Brute Force LLM**: Sending simple repeating forms to an LLM without an initial regex attempt.
- **Regex Purity**: Attempting to write a 500-line regex to handle an edge case that an LLM can solve in one sentence.
- **Blind Faith**: Assuming a regex "just works" without a programmatic confidence/validation check.
