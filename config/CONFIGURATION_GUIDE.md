# CONFIGURATION GUIDE — Ciel Skill Orchestration

This guide documents Ciel's configuration system and two-domain architecture. It is the operator reference for understanding how configuration resolves, how domains stay isolated, and how local learnings promote to global.

---

## 1. Two-Domain Architecture

Ciel operates across two always-present domains (see `domains/DOMAINS.md`):

| Domain | Path | VCS | Purpose |
| --- | --- | --- | --- |
| **Global** | `~/.ciel/` | git (`main`, linear history) | Ciel's true self — cross-project learnings, skill registry, council records, acquisition sources, self-improvement outcomes, MemPalace global partition `ciel-global`, backups. |
| **Local** | `<project>/.ciel/` | git-ignored | Project-specific context, rules, overrides, learnings, traces, checkpoints, escalation threshold, MemPalace partition `ciel-project-<hash>`. |

### Global Domain (`~/.ciel/`)
- Ciel's invariant self across sessions. Only self-improvement (gated) mutates it.
- Contains the complete skill registry, installed skills, council records, acquisition traces, self-improvement proposals, activity log, MemPalace global partition, and backups.
- Git convention: `main` branch, linear history, no force-push (Constitutional).
- Migration across machines: backup `~/.ciel/`, run install, restore backup, re-run memory health check.
- Privacy: nothing project-specific leaks here. Promoted learnings are generalized first (identifier stripping per `acquisition/HARMONIZATION.md`).

### Local Domain (`<project>/.ciel/`)
- Small, targeted, recreatable from global + project code at any time.
- Contains: `project.json` (fingerprint), `rules/`, `overrides/`, `refs/` (read-only pointers to global), `skills/` (local-only), `learnings/`, `traces/`, `checkpoints/`, `escalation.json`, `activity.log` mirror.
- Entire `.ciel/` is git-ignored (`init/GITIGNORE.md` ensures this; Ciel defensively detects and warns on accidental commits).
- Idempotent creation: re-running init on a project with existing `.ciel/` triggers integrity verification, not reset. User-edited content outside Ciel anchor blocks is preserved.
- Project fingerprint: `sha256(abspath(project_root))[:16]`. Moving the project generates a new fingerprint; Ciel can re-link via git remote or package manifest signature (user-confirmed).
- Deletion: `ciel purge <project>` drops the MemPalace partition, removes `.ciel/`, removes breadcrumbs from global promotion records, keeps the `.gitignore` entry.

---

## 2. Configuration Hierarchy

Effective config resolves in order — **later wins** (see `configuration/CONFIGURATION.md`):

```
defaults (DEFAULTS.md)
  → global overrides (configuration/global/*.config.md)
    → local overrides (configuration/local/*.config.md)
      → CLI flags
```

Constitutional floors cap the effective value — no layer may push a field below its Constitutional floor.

### File Layout
- `configuration/global/` — 9 cross-project default files, one concern each.
- `configuration/local/` — 4 project-specific override files.

Each file targets a single concern. Operators find the right knob by walking the directory rather than reading a monolithic config.

### Source of Truth
`configuration/global/*.config.md` files carry canonical YAML blocks (between `<anchor:start>` / `<anchor:end>`). Ciel reads the YAML block and produces an effective config object. Prose around the block explains rationale and boundaries.

### Global Config Files (9)
| File | Concern |
| --- | --- |
| `ciel.config.md` | Top-level: runtime prefs, telemetry, backup, auto-tune |
| `router.config.md` | Fast-path/reasoning floors, cache TTL, context budget, prompt cache, plan mode |
| `council.config.md` | Pass/reject thresholds, weights, anonymization, quorum |
| `memory.config.md` | Backend, auto-update, isolation, partition limits, fallback |
| `observability.config.md` | Log verbosity/retention, secret redaction, session summary, OTel |
| `improvement.config.md` | Per-day caps, dedup, suppression, sweep interval, regression watch, auto-tune range |
| `acquisition.config.md` | Tier timeouts, wall budget, token budget, CVE threshold, sandbox limits, min trust |
| `risk.config.md` | Risk thresholds, judge model, cost thresholds, classification weights, critical policy |
| `adapters.config.md` | Per-adapter hooks, computer use, remote control, model routing budgets |

### Local Config Files (4)
| File | Concern |
| --- | --- |
| `escalation.config.md` | Auto-detected + override escalation category (research/development/production/regulated) |
| `overrides.config.md` | Leaf-field overrides of global values for this project (Constitutional invariants cannot be overridden) |
| `project.config.md` | Auto-detected project fingerprint (id, root, language, frameworks, build/test tools, CI, license, monorepo) |
| `rules.config.md` | Codified project rules (forbidden ops, required patterns, style, testing, security, documentation) |

---

## 3. Tuning Configuration

Ciel proposes and applies config changes per `configuration/TUNING.md`.

### Categories & Gates

| Category | Examples | Gate |
| --- | --- | --- |
| **trivial** | cache TTL ±10%, fast_path_floor ±0.02 | auto (no Council) |
| **standard** | weight shifts, mid-range thresholds | Council |
| **structural** | backend swap, adapter enable/disable | Council + user |
| **constitutional** | weights.safety < 0.20, isolation_strict=false, redact_secrets=false | **rejected** |

### Auto-Apply Range
For `trivial` changes, Ciel applies within `± auto_tune_range` (default 10% or equivalent) without Council. Any change outside that range escalates to `standard`.

### Proposal Flow
1. Trigger identifies a suboptimal setting (e.g., low cache hit rate → propose higher TTL).
2. Generate proposal with evidence (before/after metrics, projection).
3. Classify category.
4. Gate (auto / Council / Council+user / rejected).
5. Apply via `configuration/global/*.config.md` anchor-block edit.
6. Post-change: observe per `self_improvement/REGRESSION_DETECTION.md`. Rollback on regression.

### User Override
User-set values pin. Ciel does not auto-tune a user-pinned field without explicit reset. Annotation `# pinned` next to the value preserves it across sweeps.

### History
Every config change is a git commit (`config_tune: <field>: <old> -> <new>`) with: trigger, evidence snapshot, council run id (if gated).

### Disabling Auto-Tune
```yaml
improvement:
  auto_tune: false
```
All tuning then requires manual user action. Default is `true`.

---

## 4. Configuration Schema

Every field is described in `configuration/SCHEMA.md` with type, default, legal range, and mutability (Constitutional / tunable / local-only). Key sections:

- **Top Level** — `version`, `runtime_prefs` (preferred, fallback_order), `telemetry` (otel_enabled, otel_endpoint).
- **Router** — `fast_path_floor` (0..1), `reasoning_floor` (0..1), `cache_ttl_minutes`, `context_budget.*`, `prompt_cache.floor`, `plan_mode.budget_tokens`.
- **Council** — `pass_score`, `weighted_pass`, `reject_threshold`, `majority_required`, `weights.*` (safety has Constitutional floor 0.20), `anonymize_stage2` (Constitutional: locked true), `stage_timeout_s`, `local_quorum_min`.
- **Acquisition** — tier timeouts, `total_wall_budget_s`, `token_budget`, `cve_threshold`, `sandbox_limits.*`, `tier1_floor`, `sandbox_retention_hours`.
- **Memory** — `backend` (mempalace|sqlite|filesystem|custom), `auto_update`, `isolation_strict` (Constitutional: locked true), `partition_size_limit_mb`, `fallback_snapshot_retention_days`.
- **Risk** — thresholds (`mid`/`high`/`critical`), `judge_confidence_floor`, `mid_judge_model`, cost thresholds, `classification_weights.*`, `critical.*` (accept_remote_approval locked false).
- **Improvement** — per-day caps, `trigger_dedup_window_hours`, `suppression_days`, `sweep_interval`, `checkpoint_stale_hours`, `regression.*`, `auto_tune`, `auto_tune_range`.
- **Observability** — `log_verbosity`, `log_retention_days`, `redact_secrets` (Constitutional: locked true), `session_summary`, `otel.*` (sampling default/critical/council).
- **Adapters** — per-runtime hooks, `computer_use` (false|preview|true), `remote_control`, `model_routing` budgets.
- **Project (local only)** — `rules`, `overrides`, `escalation.override`.

Canonical defaults are in `configuration/DEFAULTS.md` as a single YAML document for bootstrap and fallback.

---

## 5. Domain Isolation

Cross-project guarantees are a **Constitutional invariant** (`locked: true` in `domains/ISOLATION.md`).

### Guarantees
1. Project partitions never read from one another.
2. Project partitions never read from global except via explicit `lift(key)`.
3. Global partition never reads from a specific project except via `with_project(id)` scope — logged.
4. Learnings from project A never appear in project B unless promoted globally first AND generalized.
5. Deletion of a project's `.ciel/` and partition leaves no residue in other projects' data surfaces.

### Enforcement
- **Partition scoping** — MemPalace rejects cross-partition reads without explicit scope declaration.
- **Activity log** — every cross-scope operation is logged with reason, actor, keys.
- **Integrity sweep** — periodic scan for stray cross-references; violations are immediate Safety incidents.
- **Promotion stripping** — identifier removal (paths, repo names, remote URLs, internal endpoints) at promotion time.

### Leakage Vectors Defended Against
- Absolute paths in traces — stripped at promotion; hashed at storage time.
- Repo names in prompts — stripped.
- Environment secrets in stdout — redacted at capture time; never in MemPalace raw.
- Git remote URLs — stripped or hashed when crossing scopes.
- API endpoints (internal URLs) — anonymized per scope policy.

### Testing
Integrity sweep includes a probe that synthesizes a lookup with project-A identifiers against project-B partition; must return empty. Any hit is an immediate escalation.

### User Controls
- `/ciel-purge <project>` — nuke a project's data.
- `/ciel-sweep --cross-scope` — explicit cross-scope audit.
- `memory.config.md.isolation_strict` — if true (default), any cross-scope read without explicit scope declaration fails loudly.

---

## 6. Local → Global Promotion

Full protocol in `domains/PROMOTION.md`.

### Entry Criteria (minimum)
- Recurrence across ≥ 2 projects.
- Stability across ≥ N=10 invocations.
- Non-leaking.
- Non-conflicting with existing global rule.

### Pipeline
1. **Candidate identification** — `self_improvement/LOCAL_IMPROVEMENT.md` marks mature learnings.
2. **Generalization** — strip project identifiers, file paths, repo-specific jargon. Rendered form must be human-readable and self-contained.
3. **Cross-project check** — MemPalace semantic search across other project partitions for similar patterns (evidence of universality).
4. **Conflict check** — does any existing global rule/skill contradict or cover this?
5. **Council via `PROMOTION.md` scope** — `council/invocation_scopes/PROMOTION.md`.
6. **Apply** — on pass, write to appropriate global subtree (skills, rules, prompts) with provenance metadata.
7. **Cleanup** — mark local learnings `promoted:true`.

### Provenance Metadata
```yaml
promoted_from:
  projects: ["<hash1>", "<hash2>"]   # hashed identifiers, never paths
  first_seen: 2026-01-03
  recurrences: 4
  run_id: <council run id>
```
Stored in the target global file's frontmatter (for skills) or in `~/.ciel/registry/promotions.json`.

### Reversibility
A promotion is a git commit; reversing is `git revert`. Post-revert, the learning reverts to `local` status in originating projects (if still present).

### Demotion
If a promoted learning proves too project-specific after global use, `council/invocation_scopes/SKILL_CONFLICT.md` may propose demoting it back to project scope(s). Rare.

### User Initiated
`/ciel-promote <learning_id>` runs the pipeline on-demand.

---

## 7. Multi-Runtime Support

When Claude Code and Gemini CLI both invoke Ciel on the same project (`domains/MULTI_RUNTIME.md`).

### State Ownership
- `~/.ciel/` is shared — single source of truth.
- `<project>/.ciel/` is shared — single source of truth.
- Obsidian vault files are shared.
- `activity.log` is append-only and shared.

### Concurrency
- Writes use file-level locks (`flock`) on `*.json` indices.
- Obsidian provides its own concurrency primitives via the Local REST API; when available, used.
- Git commits to `~/.ciel/` serialized via file lock on `~/.ciel/.git/ciel.lock` across runtimes.
- Long-running operations (acquisition, Council) take an exclusive lock on their subsystem.

### Conflict Resolution
When two runtimes propose overlapping self-improvements:
- Second-arriving proposal is rebased on first.
- If rebase conflicts, the later proposal is queued and Ciel batches them at the next cadence sweep.

### Per-Runtime Hooks
Each adapter installs its own hooks using runtime-specific file paths (`.claude/hooks/` vs `.gemini/hooks/`). Context files (`CLAUDE.md`, `GEMINI.md`) are per-runtime and share only runtime-agnostic project rules.

### Runtime Preference (both active, no user spec)
- Computer use → Claude Code.
- Multimodal generation → Gemini CLI.
- Parallel subagents → Gemini CLI (native).
- Ultraplan → Claude Code.
- Everything else → whichever session issued the request.

### Observability
`activity.log` includes a `runtime` field per entry, so users can see which runtime did what.

---

## Reference Files

| Topic | Source |
| --- | --- |
| Configuration philosophy | `configuration/CONFIGURATION.md` |
| Canonical defaults | `configuration/DEFAULTS.md` |
| Full schema | `configuration/SCHEMA.md` |
| Tuning protocol | `configuration/TUNING.md` |
| Two-domain model | `domains/DOMAINS.md` |
| Global domain | `domains/GLOBAL.md` |
| Local domain | `domains/LOCAL.md` |
| Isolation guarantees | `domains/ISOLATION.md` |
| Promotion protocol | `domains/PROMOTION.md` |
| Multi-runtime | `domains/MULTI_RUNTIME.md` |
