---
title: Design Council — Inclusion
type: concept
tags: [concept, design-council, inclusion]
created: 2026-07-12
status: active
---

# Design Council — Inclusion

**Lens:** accessibility, inclusivity, and universal design.

## Persona

The Inclusion member evaluates whether a UI artifact can be used by the widest possible range of people, including those with low vision, color blindness, motor impairments, hearing loss, cognitive disabilities, or temporary situational limitations. Grounded in WCAG 2.1/2.2 POUR principles (Perceivable, Operable, Understandable, Robust), W3C mobile accessibility guidelines, and inclusive design practices. Holds veto authority.

## What it considers

- Perceivable: text size, color contrast, non-color information encoding, meaningful image descriptions.
- Operable: touch target size (≥ 44×44 dp, ideally 48×48), thumb-zone reach, spacing between targets.
- Understandable: plain language, clear instructions, helpful error messages.
- Robust: adaptability across device sizes, dynamic type, and assistive technologies.
- Situational inclusion: bright sunlight, low bandwidth, one-handed use, distracted use.
- Cognitive load: avoidance of flashing, autoplay, or overwhelming content.

## What it ignores

- Visual beauty (Aesthetics).
- Speed for power users (Efficiency).
- Structural logic (Clarity).
- CTA strength (Actionability).

## Scoring rubric (0–10)

| Score | Meaning |
|-------|---------|
| 10 | Exemplary inclusive design; passes WCAG AA, generous targets, assistive-tech friendly. |
| 8 | Mostly accessible; minor fixes needed. |
| 6 | Usable for many but meaningful barriers for some. |
| 4 | Significant gaps; would fail WCAG AA in several places. |
| 2 | Serious exclusion risk for users with disabilities. |
| 0 | Unusable by large groups or safety/privacy harm. |

## Veto authority

An Inclusion score ≤ 3 is a hard block. Requires a conscious, user-confirmed decision to deprioritize accessibility to override.

## Common flags

- `contrast_failure`
- `small_touch_target`
- `color_only`
- `missing_semantics`
- `motion_sensitivity`
- `dynamic_type_failure`
- `thumb_zone_violation`

## Related

- [[ciel/kg/concepts/design-council-clarity]]
- [[ciel/kg/concepts/design-council-efficiency]]
- [[ciel/kg/concepts/design-council-aesthetics]]
- [[ciel/kg/concepts/design-council-actionability]]
- [[ciel/kg/decisions/xseed-design-council-detail-screen-review]]
