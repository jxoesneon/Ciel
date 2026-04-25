---
name: research-and-intelligence
version: 1.0.0
format: skill/1.0
description: CIEL's framework for AI-powered scraping, deep research reports, and live documentation lookup.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(research|scrape|lookup).*(deep dive|exa|firecrawl|documentation)"
    confidence: 0.9
  - pattern: "deep research report"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Research & Intelligence (The Knowledge Layer)

This skill formalizes the retrieval and synthesis of external knowledge. it prioritizes cited evidence and automated data collection.

## Deep Research Workflow
1. **Breakdown**: Split the core topic into 3-5 sub-questions (Market, Tech, Competitors, Trends).
2. **Multi-Source Search**: Execute searches via Exa and Firecrawl using 3+ keyword variations per question.
3. **Deep Read**: Scrape the top 3-5 key URLs in full. do NOT rely on snippets.
4. **Synthesize**: Produce a report with inline citations and a "Confidence Score" (High/Medium/Low).

## Automated Scraping (Agent)
- **Collect**: Use `requests` or `playwright` (for JS-rendered sites). Respect `robots.txt`.
- **Enrich**: Batch items (5 per call) to Gemini Flash for scoring and summarization.
- **Store**: Sync unique items to Notion/Sheets/Supabase. Deduplicate by URL.
- **Learn**: Use a `feedback.json` to bias future AI scores based on user "Likes/Dislikes."

## Live Documentation (Context7)
- **Resolve**: Map library names (e.g., "React 19") to specific library IDs early.
- **Query**: Fetch live code snippets and configuration guides instead of relying on training data.
- **Redact**: Ensure no API keys or secrets are passed in documentation queries.

## Anti-Patterns
- **Unsourced Claims**: Adding "facts" to a report without a clickable citation.
- **Snippet-Only Synthesis**: Writing a deep dive based only on 200-character search results.
- **Rate-Limit Blindness**: Running 50 individual LLM calls for 50 scraped items.
