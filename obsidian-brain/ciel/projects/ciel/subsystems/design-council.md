---
title: Ciel — Design Council Subsystem
project_note: subsystem
type: project-note
tags: [subsystem, project, ciel]
created: 2026-07-11
status: active
---

# Ciel — Design Council Subsystem

A dedicated Council of Five for UX/UI reviews, modeled on the main Ciel Council of Five but applying design-specific lenses.

## Purpose

Evaluate user-interface and user-experience artifacts for:
- cognitive clarity and information architecture,
- accessibility and inclusivity,
- interaction efficiency and flow,
- aesthetics and trust,
- actionability and goal completion.

## Members

| Member | Lens | Veto | Analog to Ciel Council |
|--------|------|------|------------------------|
| [[ciel/kg/concepts/design-council-clarity]] | Information architecture, labeling, hierarchy, recognition over recall | No | Coherence |
| [[ciel/kg/concepts/design-council-inclusion]] | Accessibility, WCAG POUR, universal design | Yes | Safety |
| [[ciel/kg/concepts/design-council-efficiency]] | Speed, control, feedback, error prevention, thumb ergonomics | No | Efficiency |
| [[ciel/kg/concepts/design-council-aesthetics]] | Visual hierarchy, beauty, trust, brand consistency | No | Evolution |
| [[ciel/kg/concepts/design-council-actionability]] | Affordances, CTAs, progressive disclosure, error recovery | No | Capability |

## Invocation

1. Write a case description for the artifact under review.
2. Run each member as an isolated subagent with the case and their persona file.
3. Collect Stage 1 scores, rationales, and flags.
4. Synthesize findings into a cross-lens decision record (Stage 3 chairman synthesis).
5. Optionally run Stage 2 anonymous cross-review for high-stakes decisions.

## Voting math

- **Pass**: ≥ 3/5 scores ≥ 6 AND Inclusion score > 3.
- **Reject**: majority below threshold, Inclusion ≤ 3, or emergent risk in synthesis.
- **Inclusion veto**: absolute. Requires user-confirmed decision to override.

## First invocation

- **Artifact:** X-Seed detail screen (`x_seed/lib/src/features/ui/detail/detail_screen.dart`)
- **Date:** 2026-07-12
- **Decision record:** [[ciel/kg/decisions/xseed-design-council-detail-screen-review.md]]
- **Diary:** [[ciel/diary/2026-07-12-design-council-xseed-detail-screen.md]]

## Source files

- `Ciel/ciel.skill/council/DESIGN_COUNCIL.md`
- `Ciel/ciel.skill/council/members/CLARITY.md`
- `Ciel/ciel.skill/council/members/INCLUSION.md`
- `Ciel/ciel.skill/council/members/EFFICIENCY.md`
- `Ciel/ciel.skill/council/members/AESTHETICS.md`
- `Ciel/ciel.skill/council/members/ACTIONABILITY.md`

## Related

- [[ciel/projects/ciel/subsystems/core.md]] — Ciel core (identity, constitution, and main Council of Five)
- [[ciel/kg/concepts/council-subagent-invocation]] — subagent invocation rules for the main Council
