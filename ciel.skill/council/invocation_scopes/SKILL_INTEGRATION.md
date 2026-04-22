# SCOPE — SKILL_INTEGRATION

Triage: incoming acquired skill needs to join the registry.

## Trigger

`acquisition/ACQUISITION.md` has produced a harmonized candidate. Invoke the Council.

## Preamble Injected

Each member's Stage 1 prompt is prefixed:

> You are evaluating a **newly-acquired skill** for integration into Ciel's global registry. Source tier: <1|2|3>. Origin: <url|mcp|registry_id>. Harmonization report attached. Apply your lens.

## Inputs

- candidate skill (L1 default, L2 on request),
- harmonization diff,
- trust model result from `acquisition/TRUST_MODEL.md`,
- sandbox execution trace (from `acquisition/SANDBOX.md`) if Tier 2/3,
- registry metadata slice for neighbors (Capability overlap check).

## Stage 1 Output Expected

All five lenses score per `rubrics/SCORING.md`. Safety must explicitly note sandbox trace assessment.

## Stage 2

Anonymized cross-review.

## Stage 3

Chairman synthesizes. On pass, skill is:

- installed via `seed_skills/skill_installer/SKILL.md`,
- registered in `registry/REGISTRY.md`,
- committed to `~/.ciel/` with message `skill_integration: add <skill_id> (tier <n>)`,
- tagged in MemPalace `ciel/registry/<skill_id>`.

## Reject Path

- Tier 3 reject → skill discarded, origin logged, lower trust bias on that source.
- Tier 2 reject → MCP server is not auto-added; noted in acquisition.config.
- Tier 1 reject → rare; surfaces to user as a registry curation issue.

---

## Core Skill Ingestion (Local Sources)

For skills from `~/.agents/skills/` and other local runtime directories that are candidates for core Ciel integration.

## Ingestion Protocol

Unlike batch acquisition, **each core skill requires individual Council deliberation**:

```text
1. DISCOVERY
   └─→ Identify skill in ~/.agents/skills/
   └─→ Verify SKILL.md exists with basic frontmatter
   
2. ANALYSIS (Individual)
   └─→ Read full SKILL.md content
   └─→ Identify capability domain and overlap with existing skills
   └─→ Assess documentation completeness
   └─→ Note any runtime-specific dependencies
   
3. HARMONIZATION (Individual)
   └─→ Convert to Ciel format (skill/1.0)
   └─→ Add proper frontmatter (version, runtimes, triggers)
   └─→ Generate activation triggers
   └─→ Create ADAPTATION_METADATA.json
   
4. COUNCIL DELIBERATION (Required)
   └─→ Present to Council of Five with:
       - Original skill content
       - Harmonization changes (diff)
       - Capability overlap analysis
       - Proposed triggers and confidence scores
       - Backup location of original
   
5. INTEGRATION (On Council Pass)
   └─→ Install to ~/.ciel/skills/<name>/
   └─→ Register in ROUTE_REGISTRY with triggers
   └─→ Update TRIGGER_REGISTRY
   └─→ Create .bkp.zip of original in ~/.ciel/.attic/core_skills/
   └─→ Git commit: "core: integrate <skill_name> from ECC ecosystem"
   
6. REJECT (On Council Veto)
   └─→ Archive harmonization attempt
   └─→ Document rejection reason
   └─→ Return to candidate pool for future reconsideration
```

## Council Evaluation Criteria for Core Skills

| Member | Focus |
| --- | --- |
| **Coherence** | Does it align with Ciel's orchestration philosophy? |
| **Capability** | Does it fill a genuine gap? Is documentation sufficient? |
| **Safety** | Any risky operations? Sandboxing requirements? |
| **Efficiency** | Is it composable? Does it duplicate existing capabilities? |
| **Evolution** | Can it grow? Is the skill self-improving ready? |

## Command Interface

```bash
# Stage 1: Analyze candidate
./scripts/analyze-skill.sh ~/.agents/skills/<skill-name>/

# Stage 2: Prepare harmonization (dry-run)
./scripts/ingest-core-skill.sh ~/.agents/skills/<skill-name>/ --dry-run

# Stage 3: Council review (manual)
# Present to Council via council/COUNCIL.md flow

# Stage 4: Integrate (post-council approval)
./scripts/ingest-core-skill.sh ~/.agents/skills/<skill-name>/ --council-approved
```

## Current Core Skill Candidates

Located in `~/.agents/skills/` (~231 skills):

**Priority Tier (Essential capabilities):**

- `find-skills` — Skill discovery
- `gastown` — Multi-agent orchestration
- `orchestration` — General orchestration patterns
- `council` — Council deliberation

**Integration Tier (Important domains):**

- `dev-browser` — Browser automation
- `open-source-maintainer` — GitHub operations
- `security-review` — Security analysis
- `backend-patterns`, `frontend-patterns` — Domain expertise
- `testing` frameworks (multiple languages)

**Deferred Tier (Specialized/niche):**

- Domain-specific skills (healthcare, logistics, finance)
- Framework-specific skills (Django, Laravel, Spring)
- Integration-specific skills (various APIs)

See `~/.ciel/config/core_skill_backlog.md` for prioritized integration queue.
