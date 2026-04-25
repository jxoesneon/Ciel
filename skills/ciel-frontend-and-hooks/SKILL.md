---
name: ciel-frontend-and-hooks
version: 1.0.0
format: skill/1.0
description: CIEL's framework for React patterns and local rule enforcement via Hookify.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(build|review|enforce).*(react|hook|pattern|rule|hookify)"
    confidence: 0.9
  - pattern: "hookify rule"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Frontend & Hooks (The Component Layer)

This skill manages modern React development and the enforcement of local workspace rules.

## React Development Patterns
1. **Composition First**: Compose small components over complex inheritance.
2. **State Hygiene**: Use `useMemo` for sorting/filtering. Use `useCallback` for props passed to children.
3. **Fetching**: Prefer `useQuery` patterns (SWR/React Query) for SSR-safe data retrieval.
4. **Forms**: Use controlled inputs with Zod schema validation. Fail fast on bad input.

## Local Rules (Hookify)
- **Events**: Define rules for `bash` (commands), `file` (edits), `stop` (completion), or `prompt` (workflow).
- **Actions**:
  - `warn`: Show message; allow operation.
  - `block`: Prevent operation (e.g., Adding API keys to `.env`).
- **Organization**: Rules live in `.claude/hookify.{name}.local.md`. ALWAYS gitignore local rule files.

## Visual Accessibility
- **Keyboard**: EVERY interactive element must be reachable via Tab and activatable via Enter/Space.
- **Focus**: Manage focus explicitly in Modals (Restore on Close).

## Anti-Patterns
- **Fat Components**: Putting 500 lines of business logic in a single UI component.
- **Prop Drilling**: Passing state through 5+ layers instead of using Context or Zustand.
- **RegEx Loopholes**: Writing too-broad Hookify patterns (e.g., `log`) that cause noise.
