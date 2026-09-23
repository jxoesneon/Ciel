# ADR 2026-09-23 — Skill Spec Conformance and Safety Gates

**Status**: Accepted
**Date**: 2026-09-23
**Context**: `self_improvement/GROWTH_SIGNAL_20260923_LANDSCAPE_RESEARCH.md`

## Decision 1 — SKILL.md stays spec-pure; Ciel fields move to `ciel.yaml`

### Context

All 165 skills fail the Agent Skills reference validator because Ciel keeps
routing and provenance fields (`version, format, runtimes, tags, triggers,
source, dependencies, side_effects`) as top-level frontmatter keys. The spec
permits only `name, description, license, compatibility, metadata,
allowed-tools`, and `metadata` values must be strings.

### Options

1. Serialize Ciel fields into `metadata:` as JSON strings — spec-legal but
   unreadable and fragile for nested `triggers`/`dependencies`.
2. Sidecar file `ciel.yaml` beside `SKILL.md` carrying the Ciel extension
   unchanged; `SKILL.md` gains only `metadata.ciel-version` and
   `metadata.ciel-extension` pointers.
3. Drop the fields — loses the router's trigger system.

### Decision

Option 2. `SKILL.md` = the portable contract any spec-following agent can
consume; `ciel.yaml` = Ciel's routing/provenance extension. Tooling reads the
sidecar first and falls back to frontmatter for unmigrated skills.
`scripts/migrate_skill_sidecar.py --check` gates CI; `skills-ref validate`
runs on every skill in CI.

### Consequences

- Ciel skills become publishable to skills.sh and installable by any of the
  70+ spec-following agents.
- Two files per skill; the integrity manifest already hashes both.
- Seed skills, the root `ciel.skill/SKILL.md`, and guild agents are not
  migrated in this decision (internal; revisit separately).

## Decision 2 — Mandatory static scan before a skill leaves `untrusted`

### Context

Skill marketplaces are under active supply-chain attack (ClawHavoc: 1,184
malicious skills; DDIPE: doc-embedded payloads bypass 11–33% of defenses).
Ciel's trust model scores skills but never scans them.

### Decision

`scripts/scan_skills.py` wraps `skillfrisk` (offline, deterministic). A skill
whose scan reports `failed: true` cannot transition `untrusted → sandboxed`;
`risk_score` is recorded in the trust record. CI runs the scan over the whole
registry and publishes the report artifact.

### Consequences

- Blocks the known attack classes (prompt injection in docs, secret access,
  RCE patterns, Unicode hiding, MCP permission abuse) at install time.
- Does not replace sandboxing; DDIPE shows ~2.5% of payloads evade static
  analysis. The scan is the first gate, not the only one.

## Decision 3 — Paired-eval commit gate for skill mutations

### Context

SkillsBench shows self-generated skills yield no benefit on average, and the
self-evolving-skill literature attributes real gains to replay verification
with a commit gate.

### Decision

Any Council-scope skill mutation must run a paired with/without evaluation on
a small deterministic task set before Stage 1. A negative delta blocks the
proposal; results attach to the Council docket. Skills are budgeted to ≤ 3
modules.

### Consequences

- Turns `REGRESSION_DETECTION.md` from prose into an executable gate.
- Adds latency and cost per mutation; acceptable given the evidence that
  ungated self-modification does not improve outcomes.

## Decision 4 — Policy-as-code for hook risk rules

### Context

Risk regexes are duplicated literals in the Devin and Antigravity
`pre_tool_use.sh` hooks; parity drift has already occurred once.

### Decision

A single `risk/policy.yaml` declares hard-deny (non-overridable) and
soft-deny (`allow_privileged`-overridable) rules with per-tool matchers. Both
hooks load the same policy. A red-team harness fires real hook JSON events
and asserts decisions in CI.

### Consequences

- One reviewable artifact for all runtimes; adding a runtime means an adapter,
  not a copy of the rules.
- Harness makes bypasses visible before they ship.

## Deferred

- Dependency-aware skill graph retrieval (Graph-of-Skills).
- Hook → memory ingestion (claude-mem / agentmemory pattern).
- Model-diverse Council lens assignment.
- OpenTelemetry GenAI span mapping for hook events.
