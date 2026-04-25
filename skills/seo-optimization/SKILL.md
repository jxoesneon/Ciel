---
name: seo-optimization
version: 1.0.0
format: skill/1.0
description: Search visibility through technical correctness, performance, and intent-based keyword mapping.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(seo|keyword|sitemap|metadata).*(audit|plan|implement)"
    confidence: 0.95
  - pattern: "improve search visibility"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: SEO Optimization (Search Logic)

This skill formalizes the "Technical SEO" layer. it focuses on correctness and performance rather than content-stuffing gimmicks.

## The Technical Pillar
1. **Crawlability**: Enforce shallow click depth, no unintended `noindex`, and non-looping canonicals.
2. **Performance**: Hard gates for LCP < 2.5s and CLS < 0.1.
3. **Structured Data**: Implement `Organization`, `Product`, and `Article` schema (JSON-LD) where appropriate.

## On-Page Standards
- **Title Tags**: 50-60 chars. Keyword near the front.
- **Meta Descriptions**: 120-160 chars. Action-oriented value prop.
- **Heading Hierarchy**: One clear H1. H2/H3 must reflect actual content structure, not just styling.

## Keyword Mapping
- **Rule of One**: Map ONE primary search intent to ONE canonical URL.
- **Cannibalization Check**: Audit existing routes to ensure new pages don't compete with old ones.

## Anti-Patterns
- **Schema Grift**: Adding FAQ schema for content that isn't actually on the page.
- **Keyword Stuffing**: Ignoring human readability to juice bot metrics.
- **Thin Pages**: Creating programmatic "doorway" pages with near-identical content.
