---
name: ui_ux_mastery
version: 1.0.0
description: Top-of-the-line UI/UX design intelligence, component system architecture, accessibility standards (WCAG 2.2), and Core Web Vitals optimization.
triggers: [ui, ux, design system, component library, bento grid, glassmorphism, accessibility, wcag]
tags: [ui, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [documentation/SKILL.md] }
---

# ui_ux_mastery

Enforce top-of-the-line modern web design aesthetics, user-experience standards, and technical front-end performance.

## Operations

- `ui.design_system()` — generate Tailwind v4 native `@theme` tokens and shadcn/ui headless Radix component specifications.
- `ui.bento_grid(layout_schema)` — construct modular Bento Grid spatial layouts for complex SaaS dashboards.
- `ui.accessibility_audit(markup)` — verify WCAG 2.2 Level AA/AAA standards (Focus ring $\ge 3:1$, target size $\ge 24\text{px}$).
- `ui.performance_budget()` — enforce Core Web Vitals budgets ($\text{INP} < 200\text{ms}$, $\text{LCP} < 2.5\text{s}$, $\text{CLS} < 0.1$).
- `ui.agent_transparency()` — render Plan-Execute transparent layers and tool invocation state progress indicators.

## I/O Contract

```yaml
io_contract:
  input:
    op: enum
    "target_file?": string
    "component_spec?": structured_map
  output:
    result: component_code|audit_report|theme_tokens
  idempotent: true
  side_effects: [fs]
```

## Strategy

1. **Design System Engine**: Integrates Tailwind CSS v4, `@theme` native variables, and shadcn/ui headless Radix UI components.
2. **Accessibility Compliance**: Guarantees WCAG 2.2 Level AA/AAA standards (Focus appearance, touch target $\ge 24\text{px}$, high-contrast dark modes).
3. **Core Web Vitals Enforcement**: Optimizes for INP $< 200\text{ms}$, LCP $< 2.5\text{s}$, and CLS $< 0.1$.
4. **Visual Aesthetics**: Implements Bento Grid layouts, Liquid Glass backdrop-blur, dark mode HSL scales, and functional micro-interactions.
5. **Agentic UI Patterns**: Renders clear Plan-Execute transparent layers and tool invocation state progress for autonomous workflows.

## Safety

- Enforces Human-in-the-Loop Interception for destructive or irreversible UI actions.
- Mandatory Inclusion Lens veto ($\le 3.0$) for any design excluding users or violating accessibility standards.
- Evaluated by the **Design Council of Five** (`council/DESIGN_COUNCIL.md`) using `council/rubrics/UI_UX_MASTERY_STANDARDS.md`.
