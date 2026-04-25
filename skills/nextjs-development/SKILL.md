---
name: nextjs-development
version: 1.0.0
format: skill/1.0
description: Next.js 16+ development using Turbopack, Server Components, and bundle optimization.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|review).*(next.js|nextjs|turbopack|app router)"
    confidence: 0.9
  - pattern: "next dev --turbopack"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Next.js Development (The React Meta-Layer)

This skill formalizes the development of modern React applications using Next.js 16+. it prioritizes Turbopack for development and Server Components for production.

## The Turbopack Workflow
- **Dev Mode**: Default to `next dev`. Verify file-system caching is active in `.next/`.
- **HMR Integrity**: If hot updates are slow, audit for massive barrel-imports or un-optimized third-party packages.
- **Bundle Analysis**: Use the experimental `bundleAnalyzer` to find and tree-shake heavy dependencies.

## Architecture Mandates
1. **Server Components (RSC)**: Default to Server Components. Use `"use client"` ONLY for interactivity (events, state, hooks).
2. **Data Fetching**: Fetch data in Server Components via `fetch()` with appropriate `revalidate` tags.
3. **Optimized I/O**: Use `next/image` for LCP images and `next/font` for zero-CLS typography.

## Performance Gates
- **LCP**: Target < 2.5s via server-side fetching and image priority.
- **CLS**: Target < 0.1 via fixed-ratio image containers and font-optimization.
- **Cold Start**: Audit middleware and global layouts for blocking I/O that slows down TTFB.

## Anti-Patterns
- **Prop Drilling RLS**: Passing security context deep into Client Components instead of checking in RSC.
- **Barrel Import Bloat**: Importing 1 component from a 50-component index file (breaks tree-shaking).
- **Global State Overkill**: Using Redux/Zustand for data that can be managed by RSC and URL params.
