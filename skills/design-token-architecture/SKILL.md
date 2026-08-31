---
name: design-token-architecture
version: 1.0.0
format: skill/1.0
description: CIEL's framework for three-layer design token systems (primitive→semantic→component) with CSS variables and naming conventions. Advisory only.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:design"]
triggers:
  - pattern: "(design token|token architecture|css variable|design system).*(scale|naming|primitive|semantic|component)"
    confidence: 0.9
  - pattern: "(spacing|typography) (scale|token)"
    confidence: 0.9
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Design Token Architecture

This skill formalizes the *token layer* of a design system — the `ui-ux-design` skill covers style selection and `ciel-security-and-design` mentions tokens only as an anti-pattern guardrail; this one defines how to actually structure them. It is advisory only: it emits token definitions and conventions, never executes them.

## The Three-Layer Model

```
Component   (--button-bg, --card-padding)      per-component overrides
   ↑ refs
Semantic    (--color-primary, --spacing-section) purpose-based aliases
   ↑ refs
Primitive   (--color-blue-600, --space-4)       raw design values
```

- **Primitive**: Raw values with no semantic meaning (colors, sizes, radii, shadows). Change rarely — foundational.
- **Semantic**: Purpose aliases referencing primitives (`--color-primary: var(--color-blue-600)`). This is the layer you swap for theming.
- **Component**: Component-specific tokens referencing semantic layer (`--button-bg: var(--color-primary)`). Enables per-component customization without touching semantics.

**Rule**: Each layer references ONLY the layer below it. Never let a component token reach through to a raw value, and never let a semantic token hardcode a literal.

## CSS Variable Patterns

- **HSL channel triplets** for colors (`--color-primary: 222 47% 11%`) consumed via `hsl(var(--color-primary) / <alpha>)` — enables opacity control without redefining tokens.
- **Dark mode = semantic override only**: redefine semantic tokens under `.dark`; primitives and component tokens stay unchanged.
- **File organization**: `tokens/primitives.css`, `tokens/semantic.css`, `tokens/components.css`, `tokens/index.css` (imports all) — or a single file with `=== LAYER ===` comment banners.
- **W3C DTCG alignment** for JSON token sources: `{ "color": { "blue": { "600": { "$value": "#2563EB", "$type": "color" } } } }`.

## Spacing & Typography Scales

- **Spacing — 4px base**: `--space-1: 0.25rem`, `--space-2: 0.5rem`, `--space-4: 1rem`, `--space-6: 1.5rem`, `--space-8: 2rem`. Semantic aliases: `--spacing-component: var(--space-4)`, `--spacing-section: var(--space-6)`. Components reference semantics, never raw `--space-N`.
- **Typography — primitive sizes**: `--font-size-sm: 0.875rem`, `--font-size-base: 1rem`, `--font-size-lg: 1.125rem`, `--font-size-xl: 1.25rem`, `--font-size-2xl: 1.5rem`. Semantic roles: `--typography-font-heading`/`-body`/`-mono` (family), `--text-heading`/`-body`/`-caption` (size+weight+leading bundles).
- **Line-height travels with the size token**, not as a separate concern: `--font-size-lg: 1.125rem; --line-height-lg: 1.75`.
- **Keep to the scale**: arbitrary pixel values break the rhythm; add a primitive token rather than a one-off literal.

## Token Naming

```
--{category}-{item}[-{variant}][-{state}]
--color-primary              # category-item
--color-primary-hover        # category-item-state
--button-bg-hover            # component-property-state
--space-section-sm           # category-semantic-variant
```

- **Categories**: `color`, `space`, `font-size`, `font-family`, `radius`, `shadow`, `duration`, `easing`.
- **States**: `hover`, `active`, `focus`, `disabled` — suffix, never prefix.
- **Component tokens** name the *property*, not the value: `--button-bg` (good), `--button-blue` (bad — leaks implementation).

## Tailwind Integration

- Map semantic tokens into `@theme` (Tailwind v4) or `theme.extend` (v3): `colors: { primary: 'hsl(var(--color-primary) / <alpha>)' }`.
- Component tokens → component classes via `@apply` or `cva` variants.
- Validate: grep for raw hex/px in `src/` — any literal outside `tokens/primitives.*` is a violation.

## Anti-Patterns

- **Flat tokens**: `--button-primary-bg: #2563EB` with no layering — no theming, no reuse. Always migrate to three-layer.
- **Skipping the semantic layer**: Component tokens referencing primitives directly (`--button-bg: var(--color-blue-600)`) breaks theme switching.
- **Hardcoded literals in components**: Any `#hex`, `16px`, or `1rem` outside `tokens/primitives.*` is a leak.
- **Naming by value**: `--color-blue-600` is a primitive; `--button-blue` is a misnamed component token — name the property, not the value.
- **One-off spacing**: `padding: 17px` breaks the 4px rhythm — add a token or use the nearest scale step.
- **Dark mode touching primitives**: Redefining `--color-blue-600` under `.dark` cascades everywhere — override semantics only.
