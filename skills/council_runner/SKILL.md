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
2. Stage 1 — **each member is spawned as a separate subagent** (parallel
   `run_subagent` calls, one per persona). Each subagent is handed: its
   persona file path, `SCORING.md`, the artifact/evidence paths under
   review, and the member's output contract (the JSON shape in its persona
   file). Inline deliberation in the host context is not permitted — a
   council verdict without five independent subagent reports is invalid.
   Safety's subagent must assess sandbox/scan evidence explicitly.
3. Stage 2 — anonymized cross-review: resume each member subagent with the
   anonymized peer flags; members may adjust scores (identities withheld).
4. Stage 3 — `CHAIRMAN.md` synthesizes the weighted verdict in the host
   context from the five returned reports. Safety ≤ 3 is a veto
   (`VETO_CONDITIONS.md`).
5. Log the verdict as a JSONL entry in `~/.ciel/logs/council.log`.

## Subagent Dispatch Contract

Each member subagent prompt must contain:

- `persona`: absolute path to `members/<NAME>.md` — the subagent reads and
  embodies it.
- `scoring`: absolute path to `SCORING.md`.
- `evidence`: absolute paths to the artifact under review (frames, diffs,
  scan reports, rendered media stills, docs) — never URLs alone.
- `return`: the member's output-contract JSON only — `member`, `stage`,
  `score`, `rationale`, `flags`, `requests`, plus `veto` for Safety.

Members return scores only — they never edit files or run renders. Use a
read-only subagent profile where the runtime offers one.

## Inputs

Candidate artifact, harmonization diff, scan report, sandbox trace (if any),
and the applicable invocation scope
(`ciel.skill/council/invocation_scopes/*.md`).

## Artifact Review (media/visual artifacts)

When the candidate is a rendered artifact (video, image, deck):

- **Evidence format**: extract one frame per scene at its settled moment
  (plus first/last frame) to a review dir; hand members the frame paths,
  the plan/brief, the composition source, and the project's canonical docs
  (README, SKILL.md frontmatter) as the source of truth.
- **Claim-verification gate**: every on-screen metric must trace to a
  logged artifact (registry index, sanitizer run log, eval report) or be
  qualified on screen. Unanchored claims are flagged `unverified_claim`.
- **Asset-license gate**: all bundled media (music, fonts, SFX, images)
  must carry a verified redistribution license before public posting;
  attribution requirements are satisfied in-artifact (credit line) and in
  share copy.
