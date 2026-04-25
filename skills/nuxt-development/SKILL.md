---
name: nuxt-development
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Nuxt 4 development, hydration safety, and route-level rendering.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(design|build|review).*(nuxt|vue|nitro|hydration)"
    confidence: 0.95
  - pattern: "nuxt route rules"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Nuxt Development (The Vue Meta-Layer)

This skill formalizes the development of high-performance Vue applications using Nuxt 4. it focuses on hydration safety and granular rendering strategies.

## Hydration Safety (Hard Gate)
1. **Deterministic Render**: Prohibit `Date.now()`, `Math.random()`, or browser-only APIs in templates unless wrapped in `<ClientOnly>`.
2. **Client Hooks**: Move all side effects and storage reads to `onMounted()` or `import.meta.client` blocks.
3. **Route Sync**: Use Nuxt's `useRoute()` instead of `vue-router`. Prohibit using `route.fullPath` for SSR markup (fragments are client-only).

## Data Fetching Standards
- **Read**: Use `await useFetch()` for SSR-safe API reads. it prevents double-fetching on hydration.
- **Write**: Use `$fetch()` for user-triggered actions (POST/PUT/DELETE).
- **Lazy**: Use `useLazyFetch()` for non-critical data (e.g., related items) and provide explicit loading UI.

## Route-Level Strategy
Define specific behaviors in `nuxt.config.ts` using `routeRules`:
- **Static**: `prerender: true` for blogs/docs.
- **Hybrid**: `swr: true` for product catalogs.
- **App**: `ssr: false` for authenticated dashboards.

## Anti-Patterns
- **The Date Flash**: Showing `1/1/1970` on the server and the real date on the client (hydration mismatch).
- **Top-Level $fetch**: Using `$fetch` in `setup()` (blocks SSR and causes re-fetching on client).
- **Silent Loops**: Failing to provide a stable key to `useAsyncData()`, causing infinite re-fetches.
