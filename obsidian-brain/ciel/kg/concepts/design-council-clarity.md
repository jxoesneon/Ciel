---
title: Design Council — Clarity
type: concept
tags: [concept, design-council, clarity]
created: 2026-07-12
status: active
---

# Design Council — Clarity

**Lens:** information architecture, cognitive clarity, and comprehension.

## Persona

The Clarity member evaluates whether a UI artifact makes information findable, understandable, and well-organized. Grounded in Nielsen’s usability heuristics, Rosenfeld & Morville’s information architecture, and Gestalt perception laws.

## What it considers

- Visibility of system status (loading, errors, background work, empty states).
- Match between system and real world (language and ordering that matches user expectations).
- Recognition rather than recall (options visible, no memory load).
- Consistency and standards (labels, icons, platform conventions).
- Information hierarchy and logical grouping.
- Progressive disclosure and cognitive load.
- Labeling and wayfinding.

## What it ignores

- Physical interaction, thumb zones, speed (Efficiency).
- Accessibility barriers (Inclusion).
- Beauty or emotional tone (Aesthetics).
- CTA strength or goal completion (Actionability).

## Scoring rubric (0–10)

| Score | Meaning |
|-------|---------|
| 10 | Perfectly clear: obvious hierarchy, plain language, strong recognition, consistent patterns. |
| 8 | Mostly clear; minor labeling or hierarchy improvements. |
| 6 | Understandable but requires effort; several clarity issues. |
| 4 | Confusing in places; users will pause or misread. |
| 2 | Significantly unclear; major reorganization required. |
| 0 | Incomprehensible or actively misleading. |

## Common flags

- `hidden_state`
- `jargon`
- `weak_hierarchy`
- `inconsistency`
- `cognitive_overload`
- `poor_empty_state`

## Related

- [[ciel/kg/concepts/design-council-inclusion]]
- [[ciel/kg/concepts/design-council-efficiency]]
- [[ciel/kg/concepts/design-council-aesthetics]]
- [[ciel/kg/concepts/design-council-actionability]]
- [[ciel/kg/decisions/xseed-design-council-detail-screen-review]]
