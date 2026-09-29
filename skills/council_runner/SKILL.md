---
name: council_runner
description: Inline Council of Five deliberation runner. Load when a council-gated decision is needed — skill integration, self-modification, promotion, high-risk ops. Contains the five member personas (Coherence, Capability, Safety, Efficiency, Evolution), the Chairman synthesizer, scoring rubric, and veto conditions.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# council_runner

Convenes the Council of Five inline. Referenced by project rules for
council-gated decisions.

## Procedure

1. Load all five personas from `members/` — COHERENCE, CAPABILITY, SAFETY,
   EFFICIENCY, EVOLUTION.
2. Stage 1 — each member scores the candidate per `SCORING.md` with rationale
   and structured flags. Safety must assess sandbox/scan evidence explicitly.
3. Stage 2 — anonymized cross-review: members may adjust scores after seeing
   peers' flags (identities withheld).
4. Stage 3 — `CHAIRMAN.md` synthesizes the weighted verdict. Safety ≤ 3 is a
   veto (`VETO_CONDITIONS.md`).
5. Log the verdict as a JSONL entry in `~/.ciel/logs/council.log`.

## Inputs

Candidate artifact, harmonization diff, scan report, sandbox trace (if any),
and the applicable invocation scope
(`ciel.skill/council/invocation_scopes/*.md`).
