---
name: ui_ux_mastery
version: 1.0.0
description: UI/UX design governance — design systems, accessibility audits, performance budgets, agentic UX patterns.
triggers: [ui, ux, design system, accessibility, wcag, agentic ui]
tags: [ui, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [documentation/SKILL.md] }
---

# ui_ux_mastery

Generate and audit UI/UX artifacts against the Design Council standards (`council/rubrics/UI_UX_MASTERY_STANDARDS.md`).

## Operations

- `ui.design_system(spec)` — Tailwind v4 `@theme` tokens (base → semantic → component) and shadcn/ui component specs.
- `ui.bento_grid(layout_schema)` — intent-based Bento Grid layout (Hero/Utility/Micro-Data tiers) via CSS Grid.
- `ui.accessibility_audit(markup)` — WCAG 2.2 AA/AAA verification (9 new success criteria) with WCAG 3.0 forward readiness.
- `ui.performance_budget()` — Core Web Vitals field data thresholds (INP ≤ 200ms, LCP ≤ 2.5s, CLS ≤ 0.1 at 75th percentile).
- `ui.agentic_ux()` — 6-pattern control surface (Intent Preview, Autonomy Dial, Rationale, Confidence, Receipts, Audit Trail).
- `ui.thumb_zone(layout)` — dominant-hand functional area placement; destructive actions outside easy reach.
- `ui.recovery_pattern(action)` — select recovery pattern (confirmation dialog / undo window / soft delete) by reversibility.
- `ui.loading_strategy(duration_ms)` — loading state by duration (nothing < 400ms, skeleton 400ms–3s, progress 3–10s, bar > 10s).
- `ui.dark_mode()` — OKLCH four-surface luminance hierarchy (base → raised → overlay → nested).
- `ui.micro_interaction(trigger)` — spring-physics feedback (mass, stiffness, damping) in 100–400ms range.

## I/O Contract

```yaml
io_contract:
  input: { op, target_file?, component_spec?, layout_schema?, duration_ms? }
  output: { result: component_code|audit_report|theme_tokens|layout_grid|recovery_flow }
  idempotent: true
  side_effects: [fs]
```

## Standards Reference

Full 16-standard specification with research citations in `council/rubrics/UI_UX_MASTERY_STANDARDS.md`. Covers: agentic UX transparency, Core Web Vitals, WCAG 2.2/3.0, Bento Grid, Liquid Glass, Tailwind v4 tokens, shadcn/ui, micro-interactions, OKLCH dark mode, token governance, thumb-zone ergonomics, undo/cancel patterns, performance perception, OKLCH color, information architecture, competitive landscape.

## Safety

Enforces Human-in-the-Loop Interception for destructive UI actions. Inclusion Lens veto (≤ 3.0) for WCAG 2.2 violations. Action Audit Trail with reversibility status. EU AI Act transparency compliance.
