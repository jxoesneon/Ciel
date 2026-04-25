---
locked: true
---

# VETO_CONDITIONS

`locked: true` — modifications require Constitutional amendment.

Exhaustive list of conditions that trigger Safety veto (score ≤ 3).

## Hard Veto Conditions

1. **Untrusted code execution** — artifact runs arbitrary code fetched from the internet without sandboxing.
2. **Credential exfiltration** — artifact reads env vars, keychain, or `.env` to external sinks.
3. **Unbounded destructive ops** — `rm -rf`, `git reset --hard origin`, database `DROP`, etc. without explicit user consent.
4. **Permission bypass** — artifact attempts to call a tool in the host's `deny` list via an aliased name.
5. **License incompatibility** — license cannot be combined with Apache-2.0 (GPL-copyleft incompatible for direct integration; research → user decision).
6. **No origin provenance** — Tier 3 acquisition with no identifiable source URL and no sandbox trace.
7. **Known-vulnerable dep** — declared dependency has a CVE above `configuration/global/acquisition.config.md.cve_threshold`.
8. **Irreversible irreversible** — artifact performs an operation with no rollback path (e.g., publish to immutable registry, send email to external parties) without a guard clause.
9. **Council / Constitution attack** — artifact modifies `council/` or `core/CONSTITUTION.md` outside amendment procedure.
10. **PII handling without policy** — artifact processes PII with no declared handling policy.

## Soft Vetoes

Score 4–5 (not a veto, but capped by Accuracy-as-Ceiling):

- Borderline license that may be acceptable with attribution.
- Dependency with medium CVE that has a patch available.
- Destructive op with clear guard clauses.

## Evidence Requirement

A Safety veto must cite at least one concrete piece of evidence (file snippet, dep list entry, origin URL mismatch, etc.). "Gut feel" vetoes are disallowed; the Chairman rejects them as malformed.

## Over-ride Procedure

1. Council rerun with rework addressing each veto condition.
2. If still vetoed, user may propose Constitutional amendment (rarely appropriate).
3. Amendment requires full Council re-vote + explicit user confirmation.
