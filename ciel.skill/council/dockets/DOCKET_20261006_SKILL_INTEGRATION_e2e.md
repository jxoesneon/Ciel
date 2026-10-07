# DOCKET — SKILL_INTEGRATION: e2e (tester-army)

Date: 2026-10-06
Scope: `council/invocation_scopes/SKILL_INTEGRATION.md`
Candidate: `e2e` — agentic end-to-end testing skill bundle

## Provenance

- Source tier: 3 (web/git, uncurated source)
- Origin: https://github.com/tester-army/e2e/tree/main/skills/e2e
- Upstream commit: b7ba0097abe950b6cd3da25c45e58087fc77b668 (2026-10-06)
- License: Apache-2.0
- Artifacts: SKILL.md + 8 reference docs (~125 KB, markdown only)
- Fetch hashes (sha256) recorded in `ciel.yaml` of the installed bundle

## Trust Gate / Sandbox Assessment

Pure documentation bundle — no executable code, scripts, or hooks. Content
scan for injection markers (`ignore instructions`, credential shapes, shell
exfiltration, destructive ops): clean; single `base64` hit is legitimate
HTTP-basic-auth documentation. Skill's own rules mandate secrets stay out of
test code via `credentials`/`secrets` config namespaces. Runtime side effects
(npm install of `e2e`, Playwright browsers, model calls) are the tool's
documented behaviour, deferred to execution time and user-controlled.

## Harmonization Report

```yaml
harmonization_report:
  diffs:
    - path: SKILL.md
      change: "frontmatter extended: +license, +metadata{ciel-version, ciel-extension, upstream-repo, upstream-commit}; body verbatim"
    - path: ciel.yaml
      change: "created: schema 1 extension (runtimes, tags, triggers, source, dependencies)"
  unknown_tags: []
  warnings:
    - "name 'e2e' is generic; direct trigger anchored to exact-match only, functional triggers require tester-army/e2e-framework context"
```

## Stage 1 — Five Lenses

| Member | Score | Rationale |
| --- | --- | --- |
| Coherence | 8 | Aligns with quality-guild mandates: verification-first (`expect` after every `agent.act`), no sleeps, recorded replay for determinism. Fits Ciel orchestration philosophy. |
| Capability | 9 | Fills a genuine gap: no installed skill covers the `e2e` agentic runner — its config, replay cache, MCP server, explore/bug-bash workflows. `e2e-and-visual-verification` is generic Playwright/POM + design; complementary, not duplicative. Documentation depth is exemplary (8 topic files). |
| Safety | 8 | No hard-veto condition present. Markdown-only artifact, clean scan, Apache-2.0, full provenance (URL + commit + per-file sha256). Noted: CLI telemetry exists upstream but is opt-out documented; secret-redaction guidance included in the skill itself. |
| Efficiency | 7 | Whole-skill acquisition rather than composition — justified: the bundle is upstream-maintained and self-contained; fragmenting it would lose fidelity and forfeit the re-fetch path. Skill is internally composable (topic-scoped reference loading keeps L1 small). |
| Evolution | 8 | Actively maintained upstream (commit same-day), versioned npm releases, skill ships inside the repo — clean re-fetch/diff path for updates. |

## Chairman Synthesis

Weighted: 0.20(8) + 0.20(9) + 0.25(8) + 0.15(7) + 0.20(8) = **8.05**

- Majority: 5/5 members ≥ pass_score (6)
- Weighted ≥ 6.5: yes
- Safety ≥ 6: yes — normal synthesis applies

## Verdict: PASS — integrate

Actions on pass:

- Install to `~/.ciel/skills/e2e/` (SKILL.md, ciel.yaml, references/)
- Register route `route_e2e_agentic_v1` in ROUTE_REGISTRY.md
- Add triggers to TRIGGER_REGISTRY.md
- Commit: `skill_integration: add e2e (tier 3)`

Flags: `trust_state: untrusted` pending paired eval (consistent with prior
Tier-3 acquisitions: impeccable, ui-ux-pro-max, emil-design-eng).
