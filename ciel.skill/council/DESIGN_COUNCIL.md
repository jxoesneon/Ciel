# DESIGN COUNCIL — Council of Five for UX/UI Review

A dedicated Council of Five for evaluating user-interface and user-experience artifacts. It mirrors the governance structure of the main Ciel Council but applies design lenses instead of system-integration lenses.

## Members

Five lenses, each a dedicated evaluator:

- **Clarity** — information architecture, cognitive clarity, comprehension (`members/CLARITY.md`)
- **Inclusion** — accessibility, inclusivity, universal design, veto authority (`members/INCLUSION.md`)
- **Efficiency** — interaction speed, control, flow, ergonomics (`members/EFFICIENCY.md`)
- **Aesthetics** — visual hierarchy, emotional resonance, trust (`members/AESTHETICS.md`)
- **Actionability** — affordance, conversion, goal completion (`members/ACTIONABILITY.md`)

## Design Council Mapping to Ciel Council

| Ciel Council | Design Council | Shared Concern |
| ------------ | ---------------- | ---------------- |
| Coherence | Clarity | harmony, consistency, fitting patterns |
| Capability | Actionability | can the user/agent accomplish the goal |
| Safety | Inclusion | risk of harm/exclusion, veto authority |
| Efficiency | Efficiency | leanness, speed, cost of use |
| Evolution | Aesthetics | long-term craft, growth of quality and trust |

## Three Stages

**Stage 1 — Independent scoring (parallel).** Each member evaluates the artifact against their lens using the rubric in their member file. Output: `{ score: 0..10, rationale, flags, requests }`.

**Stage 2 — Cross-review (anonymous).** Each member sees the four peer outputs without attribution. They may revise. Output: `{ score, rationale, challenge_of: [anon_id], delta_reason }`.

**Stage 3 — Chairman synthesis.** The orchestrator synthesizes the five perspectives into a single prioritized recommendation, ordered by severity and aligned with the user’s goal.

## Voting Math

- **Pass**: ≥ 3/5 final scores ≥ 6 AND Inclusion score > 3 (no veto).
- **Reject**: majority below threshold, Inclusion ≤ 3, or emergent risk in synthesis.
- **Deadlock**: scores clustered around threshold ± 1.0. Escalate to the user with the conflicting rationales.

## Inclusion Veto — Absolute

An Inclusion score ≤ 3 is a hard block regardless of other votes. It signals that the design excludes a meaningful group of users or creates safety/privacy harm for vulnerable users. Overriding requires a conscious, user-confirmed decision to deprioritize accessibility.

## Quorum

- Minimum 3 non-abstaining members for any decision.
- Inclusion must be present and non-abstaining.
- If Inclusion is absent/timed-out, the run is re-attempted once; second failure escalates.

## Invocation

Convene by running each member as an isolated subagent with their persona file and a shared case description. The orchestrator then performs Stage 3 synthesis. The first invocation of the Design Council was for the X-Seed detail screen on 2026-07-11.
