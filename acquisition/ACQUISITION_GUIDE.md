# Ciel Acquisition Guide

## Overview

The acquisition system fills capability gaps by discovering, validating, and registering new skills through a tiered pipeline. It is invoked by the router's acquisition path when gap detection fires.

## Tiered Pipeline

```
gap_description
    │
    v
Tier 1: Curated Registry (timeout 10s, floor 0.5)
    │ miss
    v
Tier 2: MCP Discovery (timeout 30s)
    │ miss
    v
Tier 3: Web Extraction + Synthesis (timeout 120s)
    │
    v
Composition / Harmonization
    │
    v
Trust Gate + Sandbox
    │
    v
Council of Five (SKILL_INTEGRATION scope)
    │ pass
    v
Register → Route → Execute
    │ fail
    v
Discard + Log + Source trust reduced
```

### Tier 0 — Local Skill Discovery
Ingest existing skills from runtimes before attempting external acquisition.

### Tier 1 — Curated Registry (`TIER_1_REGISTRY.md`)
- Fastest, highest trust (origin tier bias = 1.0).
- Lookup in `~/.ciel/registry/ROUTE_REGISTRY.json` and `SKILL_INDEX.json`.
- Metadata validation + light sandbox.
- Timeout: 10 seconds.

### Tier 2 — MCP Server Discovery (`TIER_2_MCP.md`)
- Medium trust (origin tier bias = 0.8).
- Discovers tools from available MCP servers.
- Currently available MCP servers:
  - **mempalace-rs** (20 tools) — memory palace, knowledge graph, diary
  - **sequential-thinking** (1 tool) — step-by-step reasoning
  - **mcp-playwright** (20 tools) — browser automation
- Full schema validation + sandbox with declared side-effects.
- Timeout: 30 seconds.

### Tier 3 — Web Extraction + Synthesis (`TIER_3_WEB.md`)
- Lowest trust (origin tier bias = 0.4).
- Web search + LLM extraction to synthesize a new skill.
- Full sandbox, network denied by default, isolated filesystem.
- Timeout: 120 seconds.

## Composition vs Acquire-Whole

Ciel prefers composing new skills from existing fragments + one or two new primitives over importing monolithic blobs. See `COMPOSITION.md`.

A composition skill references its component skills:
```yaml
id: "deploy_static_site/SKILL.md"
version: 1.0.0
composes:
  - git/SKILL.md
  - shell/SKILL.md
  - api_client/SKILL.md
flow: |
  1. git/SKILL.md: checkout
  2. shell/SKILL.md: run build
  3. api_client/SKILL.md: upload to CDN
```

Benefits: smaller footprint, inherited reliability, component changes auto-propagate.
Risks: cascade failures → trigger more aggressive outcome scoring.

## Harmonization

Every acquired artifact passes harmonization before Council. See `HARMONIZATION.md`.

| Dimension | Action |
|---|---|
| Frontmatter | Ensure `name`, `version`, `description`, `triggers`, `tags`, `runtime_compatibility` |
| Naming | Path converted to `<domain>/SKILL.md` |
| Tags | Mapped into Ciel's taxonomy; unknown tags flagged |
| I/O contract | Expressed as `io_contract` per `registry/SCHEMA.md` |
| Prose style | Short, direct, Ciel's tone |
| Dependencies | Declared in `dependencies` — none implicit |
| Source attribution | `source.origin` populated with URLs + fetch hashes |
| License | Added as `source.license` using SPDX id |

Harmonization does NOT change behaviour, remove attribution, or alter licenses.

## Trust Model

Untrusted → sandboxed → validated → promoted. See `TRUST_MODEL.md`.

| State | Description |
|---|---|
| `untrusted` | just arrived, harmonized, never executed |
| `sandboxed` | executed in isolation with synthetic inputs; trace captured |
| `validated` | Council-approved; registered; under observation |
| `promoted` | matured: confidence high, used repeatedly, can be a composition component |
| `suspect` | observed anomalies; under review |
| `deprecated` | scheduled for removal |

Trust score formula:
```
trust = 0.4 * origin_tier_bias
      + 0.2 * sandbox_pass_rate
      + 0.2 * production_success_rate
      + 0.1 * council_pass_count
      + 0.1 * age_bonus (log-scaled, capped)
```

Demotion: `validated` → `suspect` → `deprecated` → removal.

## Sandbox Protocol

Isolated execution of unvalidated skill candidates. See `SANDBOX.md`.

### Isolation Layers
1. **Filesystem** — chroot / bind-mount to `~/.ciel/sandbox/<run_id>/`
2. **Network** — default deny; allowlist only declared domains
3. **Shell** — restricted PATH; no privileged binaries
4. **Resource limits** — CPU 50%, memory 512MB, wall 60s
5. **Secrets** — mock tokens only

### Backends (in order of preference)
1. Docker / Podman
2. macOS Seatbelt (sandbox-exec)
3. Firejail / bubblewrap (Linux)
4. chroot (last resort)

If no isolation backend is available, Ciel refuses Tier 3 acquisitions and escalates.

### Trace Capture
Sandbox traces (commands, stdout/stderr snippets, exit codes, network attempts, fs writes, anomalies) are attached to Council input.

### Cleanup
Sandbox directory retained for 48 hours then purged. Traces persist in MemPalace.

## Council Triage

Acquired skills must pass the Council of Five under the `SKILL_INTEGRATION` invocation scope before registration. The Council evaluates:
- **Coherence** — does it fit the repository?
- **Capability** — genuine expansion vs redundancy?
- **Safety** — risk vectors (VETO authority)
- **Efficiency** — leanness, bloat, performance
- **Evolution** — growth trajectory

On Council reject: artifact retained at `~/.ciel/.attic/acquired_rejected/<run_id>/` for audit; source trust lowered.

## Registration

On successful Council pass:
1. New skill installed at `~/.ciel/skills/<id>/`
2. Trigger generation via `TRIGGER_GENERATOR.md`
3. Registry updated (`ROUTE_REGISTRY.json`, `TRIGGER_REGISTRY.json`, `SKILL_INDEX.json`)
4. MemPalace embedding computed
5. Git committed

## Budgets

| Budget | Value |
|---|---|
| Tier 1 timeout | 10s |
| Tier 2 timeout | 30s |
| Tier 3 timeout | 120s |
| Total wall budget | 300s |
| Token budget | 80,000 |

Exceeded budgets degrade to the next tier or escalate to user.
