---
name: market-research
version: 1.0.0
format: skill/1.0
description: Evidence-first research for competitive analysis, fund diligence, and market sizing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(research|analyze).*(market|competitor|investor|category|trend)"
    confidence: 0.95
  - pattern: "tam/sam/som"
    confidence: 1.0
---

# CIEL ADAPTATION: Market Research (Strategic Discovery)

This skill formalizes "Decisional Research"—output designed to inform business strategy rather than provide generic summaries. It mandates source attribution and contrarian evidence.

## Research Standards
1. **Source or Label**: Every claim MUST have a source link or be explicitly labeled as an `ESTIMATE`.
2. **Decision-Oriented**: Translate all findings into a concrete decision (e.g., "Fit: Yes/No", "Action: Build/Buy").
3. **Pessimism Mandate**: The Orchestrator MUST actively search for contrarian evidence and downside risks (the "Bear Case").

## Research Modes
- **Investor Diligence**: Focus on fund stage, check size, relevant portfolio companies, and red flags.
- **Competitive Analysis**: Focus on product reality and positioning gaps rather than marketing copy.
- **Market Sizing**: Combine top-down reports with bottom-up acquisition assumptions.

## Output Structure
1. **Executive Summary**: The single biggest insight.
2. **Key Findings**: Sourced facts.
3. **The Bear Case**: Why this might fail.
4. **Recommendation**: Clear NEXT_ACTION.
5. **Sources**: Links/Docs.

## Anti-Patterns
- **Research Theater**: Producing 2000 words of generic analysis that doesn't drive a decision.
- **Amnesiac Research**: Failing to check internal `MemPalace` or `docs/research/` for prior work.
- **Vibes-Sourcing**: Claiming "industry consensus" without a single credible link.
