---
title: Design Council — Efficiency
type: concept
tags: [concept, design-council, efficiency]
created: 2026-07-12
status: active
---

# Design Council — Efficiency

**Lens:** interaction speed, control, and flow.

## Persona

The Efficiency member evaluates whether a UI artifact lets users complete their tasks quickly, confidently, and with minimal friction. Grounded in Nielsen’s heuristics for user control, error prevention, and flexibility; Fitts’s Law; Hick’s Law; and mobile ergonomics research.

## What it considers

- Feedback and responsiveness.
- User control and freedom (undo, cancel, back out).
- Error prevention and confirmation discipline.
- Thumb-zone ergonomics and one-handed use.
- Steps to goal and input efficiency.
- Flexibility for experts and novices.
- Performance perception (skeletons, placeholders, optimistic UI).

## What it ignores

- Structural understandability (Clarity).
- Inclusivity (Inclusion).
- Beauty or trust (Aesthetics).
- CTA strength (Actionability).

## Scoring rubric (0–10)

| Score | Meaning |
|-------|---------|
| 10 | Frictionless: fast feedback, excellent ergonomics, minimal steps, strong error prevention. |
| 8 | Mostly efficient; a few extra taps or weak feedback moments. |
| 6 | Usable but noticeably slow or cumbersome; several friction points. |
| 4 | Inefficient; users will feel annoyance or repeat steps. |
| 2 | Very high friction; core tasks take far more effort than necessary. |
| 0 | Broken flow; users cannot reliably complete the task. |

## Common flags

- `missing_feedback`
- `extra_steps`
- `poor_thumb_zone`
- `no_undo`
- `error_prone`
- `slow_perception`
- `over_confirm`

## Related

- [[ciel/kg/concepts/design-council-clarity]]
- [[ciel/kg/concepts/design-council-inclusion]]
- [[ciel/kg/concepts/design-council-aesthetics]]
- [[ciel/kg/concepts/design-council-actionability]]
- [[ciel/kg/decisions/xseed-design-council-detail-screen-review]]
