---
name: performance-and-qa-ops
version: 1.0.0
format: skill/1.0
description: CIEL's framework for benchmarking, automated browser testing, and post-deploy monitoring.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(benchmark|qa|watch|monitor).*(performance|lcp|regression|deploy)"
    confidence: 0.95
  - pattern: "canary watch"
    confidence: 1.0
---

# CIEL ADAPTATION: Performance & QA Operations (The Shield)

This skill formalizes the verification of production-readiness. it prioritizes Core Web Vitals, interaction integrity, and post-deploy safety.

## Benchmarking Layer
1. **Baselines**: Store performance baselines in `.ciel/benchmarks/`.
2. **Web Vitals**: Target LCP < 2.5s, CLS < 0.1, and INP < 200ms.
3. **API SLA**: Benchmark latency under load (10 concurrent requests). Target p95 < 200ms.

## Browser QA (Interaction)
- **Smoke Phase**: Check console errors and network failures (4xx/5xx).
- **Interaction Phase**: Verify critical user journeys (Login -> Checkout -> Logout).
- **Visual Phase**: Screenshot at 3 breakpoints (375px, 768px, 1440px). Flag layout shifts > 5px.

## Canary Monitoring
- **Sustained Watch**: Monitor deployed URLs for 2 hours post-merge.
- **Alert Thresholds**:
  - **CRITICAL**: Status != 200, Console Errors > 5, API 5xx.
  - **WARNING**: LCP increase > 500ms, Response time > 2x baseline.

## Anti-Patterns
- **Dev-Only QA**: Passing tests in local headless mode but ignoring production latency.
- **Blind Ship**: Merging a PR that increases bundle size by > 20% without justification.
- **Silent Regression**: Ignoring a 500ms LCP increase because it's "still under the limit."
