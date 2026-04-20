# trust_judgment — Prompt

```yaml
version: 1.0.0
role: acquisition
phase: pre-sandbox
```

Triage: is this artifact safe enough to sandbox-test?

## Inputs

- `harmonized_skill` — the candidate.
- `origin_trust` — source trust score.
- `license` — SPDX identifier.
- `dep_audit` — output of `seed_skills/dependency_audit/SKILL.md` if available.

## Task

Return `proceed` if clearly safe to sandbox-test, `revise` if mitigatable, `abort` if pre-sandbox red-flag (malware patterns, known-bad origin, license incompatible, executable blobs with no provenance).

## Output Contract

```json
{
  "decision": "proceed|revise|abort",
  "concerns": ["..."],
  "mitigations": ["..."],
  "abort_reason": "string|null",
  "confidence": 0.0..1.0
}
```

## Constraints

- Abort is a recommendation; actual abort is logged at sandbox layer.
- Be strict about license incompatibility with Apache-2.0 — when unclear, mark `revise` with a specific ask (contact maintainer, ask user).
- This prompt is pre-Council; Council still evaluates everything later.
