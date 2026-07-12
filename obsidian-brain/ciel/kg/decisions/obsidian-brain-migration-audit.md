---
title: Council Audit — Obsidian Brain Migration
type: decision
tags: [decision, adr, audit, ciel]
project: ciel
decision_date: 2026-07-08
created: 2026-07-08
status: adopted
run_id: run-obsidian-audit-1
---

# Council Audit — Obsidian Brain Migration

Comprehensive Council of Five audit of the `Obsidian` branch implementation that replaces the `mempalace-rs` memory stack with an Obsidian-based brain.

## Subagent Audit Attempt

Per the updated Council invocation rule, Ciel attempted to re-run this audit with each member as a separate `claude -p` subagent. The `claude` CLI is installed but not authenticated in this session (`Not logged in · Please run /login`), so no subagent runtime was available. This audit therefore falls back to monolithic synthesis by the Chairman, using the same lens rubrics and scoring weights as the subagent protocol. A reusable subagent runner is preserved at `scripts/council/run-subagent-audit.mjs` for re-execution after authentication.

## Case

**Artifact under review:**

- `obsidian-brain/` — Obsidian vault starter pack.
- `ciel.skill/memory/backends/obsidian/` — `CielMemoryBackend` adapter.
- `skills/obsidian-memory/SKILL.md` — skill wrapper.
- `scripts/obsidian/agentic-loop.mjs` — agentic loop orchestrator.
- `tests/obsidian-memory/adapter.test.mjs` — verification tests.
- `docs/OBSIDIAN_BRAIN.md` — setup guide.
- `ciel.skill/configuration/global/memory.config.md` — backend configuration update.

**Constraints:**

- Local-first, privacy-preserving, human-auditable memory.
- Must implement the existing `CielMemoryBackend` interface.
- Must not break existing `mempalace-rs` fallback until explicitly switched.

**Success criteria:**

- Passes all five Council lenses.
- Tests pass against a mock Obsidian REST API.
- Self-test can verify the live stack.
- Agentic loop can turn goals into persisted tasks and notes.

## Stage 1 — Independent Scoring

### Coherence (Repository Harmony)

**Score:** 8 / 10

**Rationale:** The implementation fits Ciel's existing conventions. The `obsidian-memory` skill uses the same `skill/1.0` frontmatter format as sibling skills. The adapter implements the abstract `CielMemoryBackend` interface defined in `seed_skills/mempalace_manager/SKILL.md`. The configuration block follows the `memory.config.md` pattern. New directories (`obsidian-brain/`, `scripts/obsidian/`) are additive and do not collide with existing naming.

**Flags:**

- `doc_style_mismatch` — `obsidian-brain/` sits at the repo root rather than under `.ciel/` or `~/.ciel/`, which is a minor departure from the existing `~/.ciel/` convention. Acceptable because the user wants an openable Obsidian vault, but should be documented.

**Requests:** None.

### Capability (Genuine Expansion vs Redundancy)

**Score:** 9 / 10

**Rationale:** The artifact fills a real and currently unmet gap: a human-readable, agent-agnostic, local-first memory substrate. It replaces `mempalace-rs` (which the user explicitly wants to move away from) and adds Obsidian-native semantic search, knowledge graph traversal, and an agentic loop. No existing Ciel skill provides this combination.

**Flags:**

- `fills_gap:obsidian-memory` — local markdown-native memory with AI agent integration.
- `ecosystem_bridge` — connects Ciel to the Obsidian plugin ecosystem.

**Requests:** None.

### Safety (Risk Vectors)

**Score:** 7 / 10

**Rationale:** The design is local-first and auditable. API credentials are read from environment variables (`OBSIDIAN_API_KEY`), not hardcoded. The default endpoint is `127.0.0.1`. Memory writes are plain markdown, so every mutation is visible to the user. However, the adapter can spawn `npx` subprocesses to start `obsidian-hybrid-search`, and the REST client currently defaults to HTTP (with HTTPS supported but not rigorously validated). The agentic loop can run arbitrary shell commands when `--execute` is passed, which is gated by dry-run by default.

**Flags:**

- `subprocess_spawn` — `obsidian-hybrid-search` and `obra/knowledge-graph` may be auto-started via `npx`.
- `network_local_only` — default configuration is localhost; good.
- `secrets_env` — API key is env-driven; good.

**Requests:**

- `L2` — review subprocess spawning and add timeouts/kill switches.
- `sandbox_trace` — run the adapter self-test in a sandbox before promotion.

### Efficiency (Leanness, Bloat, Performance)

**Score:** 7 / 10

**Rationale:** The adapter is ~500 lines and well-scoped. The skill documentation is concise. The vault uses plain markdown with YAML frontmatter, avoiding binary overhead. Some duplication exists between `docs/OBSIDIAN_BRAIN.md` and `obsidian-brain/README.md`, and the agentic loop script could be trimmed. The dependency footprint is minimal (`js-yaml`).

**Flags:**

- `context_bloat` — minor duplication between setup guide and vault README.
- `l1_oversize` — `obsidian-memory/SKILL.md` is slightly longer than the typical L1 skill summary; acceptable for a foundational system skill.

**Requests:** None.

### Evolution (Growth Trajectory)

**Score:** 9 / 10

**Rationale:** The migration is catalytic. It turns Ciel's memory from a closed Rust binary into an open, extensible markdown vault that any AI agent can read and write. The structure (`raw/` → `wiki/`, `ciel/kg/`, `ciel/projects/`) generalizes to future agent memory patterns. The agentic loop can be reused for any goal, not just memory migration.

**Flags:**

- `catalyst` — unlocks a whole class of local-first, markdown-native agent integrations.
- `generalizable` — the vault structure and agentic loop are reusable patterns.
- `ecosystem_bridge` — brings Ciel into the Obsidian/MCP ecosystem.

**Requests:** None.

## Stage 2 — Cross-Review (Anonymous)

**Summary of peer challenges:**

- **Coherence → Safety:** The root-level `obsidian-brain/` vault is acceptable for user convenience, but Safety wants a documented path to move it to `~/.ciel/obsidian-brain/` if the user prefers.
- **Efficiency → Capability:** The capability is large enough that the skill's L1 summary should be split into a shorter version with L2 details available on demand.
- **Evolution → Efficiency:** The agentic loop is powerful but could be a separate skill in the future to keep the memory skill focused.
- **Safety → Coherence:** The `npx` subprocess spawning is not a Ciel convention; prefer a documented, opt-in service start rather than auto-start.
- **Capability → Safety:** Agrees that auto-start of services is a convenience feature but should be explicitly gated.

**Revised scores after cross-review:** No material changes. All members maintain their Stage 1 scores with the above notes recorded as mitigations.

## Stage 3 — Chairman Synthesis

**Weighted score:** 8.0 / 10

Calculation per `council/rubrics/SCORING.md`:

```text
0.20 * Coherence    8  = 1.60
0.20 * Capability   9  = 1.80
0.25 * Safety       7  = 1.75
0.15 * Efficiency   7  = 1.05
0.20 * Evolution    9  = 1.80
                       ------
Weighted total          8.00
```

**Majority:** 5/5 members scored ≥ 6. Safety score is 7 (> 3), so no veto applies.

**Verdict:** **PASS with mitigations.** The Obsidian brain migration is approved for adoption on the `Obsidian` branch. The implementation is sound, fills a strategic gap, and aligns with Ciel's long-term evolution toward local-first, auditable agent memory.

## Required Actions Before Promotion to Default Backend

1. **Harden subprocess spawning.** Add explicit timeouts, process cleanup, and an opt-in flag for auto-starting `obsidian-hybrid-search` / `knowledge-graph` services.
2. **HTTPS / certificate handling.** Default to HTTP for local development; document how to pin the Local REST API certificate for HTTPS if the user enables it.
3. **CI self-test.** Add `npm run self-test` to the branch verification pipeline.
4. **Data migration path.** Document how to export existing MemPalace partitions into `obsidian-brain/ciel/` folders.
5. **Rate limiting / concurrency guards.** Add a simple lock file or semaphore in the agentic loop to prevent concurrent writes to the same note.
6. **L1 trim.** Produce a shorter L1 summary of `skills/obsidian-memory/SKILL.md` for the router context budget.

## Adoption Decision

- **Approved on branch:** `Obsidian`
- **Default backend status:** Adopted for this branch; requires user confirmation before becoming the global default.
- **Fallback:** Keep `mempalace-rs` intact on other branches until the user explicitly switches `memory.config.backend` to `custom`.
- **Next review:** After the required mitigations are completed and the user has run the live self-test against a real Obsidian vault.

## Related

- [[ciel/kg/decisions/]]
- [[ciel/projects/ciel/ciel]]
- [[docs/OBSIDIAN_BRAIN]]
- [[_CLAUDE.md]]
