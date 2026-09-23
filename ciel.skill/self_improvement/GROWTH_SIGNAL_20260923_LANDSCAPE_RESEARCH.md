# GROWTH SIGNAL: Landscape Research — Agent Skill Ecosystems, Governance, and Safety

**Date**: 2026-09-23
**Trigger**: `scheduled_sweep` / user-directed deep research
**Category**: Strategy, Interoperability, Security, Self-Improvement

---

## 1. Scope

Survey of systems comparable to Ciel — portable skill layers, multi-model
governance, agent memory, self-improving skill libraries, hook-based
guardrails, skill supply-chain security, and observability standards — with
each finding checked against Ciel's own code where a claim was testable.

## 2. Where Ciel already leads

- **Multi-lens governance with an absolute safety veto.** The Council of Five
  (`council/COUNCIL.md`) anticipates what `llm-council.dev` and
  `rachittshah/llmcouncil` now productize: parallel independent scoring,
  anonymized cross-review, chairman synthesis. The *Language Model Council*
  paper (arXiv 2406.08598) confirms anonymized peer ranking is more robust and
  more human-consistent than any single judge.
- **Out-of-model enforcement.** Hook-level gating (`hooks/*/pre_tool_use.sh`)
  is the industry consensus (APort guardrails, sentinel-hooks,
  karanb192/claude-code-hooks): prompt-level guardrails can be reasoned around;
  a `PreToolUse` process cannot.
- **Tiered trust and an integrity manifest.** Mainstream skill managers
  (skills.sh / vercel-labs/skills, agent-skills-cli) install with no trust
  model at all.

## 3. Findings and evidence

### 3.1 Interoperability — Ciel skills fail the Agent Skills validator

- Anthropic released Agent Skills as an open standard (agentskills.io,
  2025-12-18). Allowed frontmatter keys: `name, description, license,
  compatibility, metadata, allowed-tools`. `metadata` is a string→string map.
- **Verified locally:** `npx -y skills-ref validate skills/double-loop` →
  "Unexpected fields in frontmatter: dependencies, format, runtimes, source,
  tags, triggers, version." All 165 skills carry these keys (21 also carry
  `side_effects`).
- Positive: 0/165 names violate the spec; average SKILL.md is 59 lines
  (spec recommends < 500); one skill exceeds 500 lines.
- Ecosystem reach: the skills CLI targets 70+ agents; marketplaces list
  175k+ skills. Non-conformance excludes Ciel from all of it.

### 3.2 Supply-chain security — the dominant active threat

- ClawHub (OpenClaw marketplace): 341 malicious skills found Feb 2026
  (11.9% of inventory), rising to 1,184 ("ClawHavoc") by May 2026,
  distributing the Atomic macOS Stealer (CSA research notes, 2026-06).
- DDIPE (arXiv 2604.03081): payloads embedded in code examples and config
  templates inside skill docs; 11.6–33.5% bypass across four frameworks,
  while explicit-instruction attacks score 0% under strong defenses.
  Static analysis catches most; 2.5% evade both static analysis and alignment.
- OWASP MCP Top 10 codifies tool poisoning (MCP03) and supply-chain
  tampering (MCP04). Shai-Hulud campaign shipped 170 packages with *valid*
  SLSA L3 provenance — provenance alone is insufficient.
- Tooling: `skillfrisk` (offline, ms, zero-false-positive corpus, capability
  delta on `allowed-tools`/shell/network), `nyuwayskillscanner`
  (ALLOW/REVIEW/BLOCK, SARIF), `SkillScan` (static + LLM dry-run + sandbox
  honeypots). **Verified locally:** `uvx skillfrisk scan skills/double-loop
  --json` → `risk_score: 0, failed: false`.
- Ciel gap: `acquisition/TRUST_MODEL.md` scores trust but has no static scan
  gate before `untrusted → sandboxed`.

### 3.3 Self-improvement — self-generated skills need a commit gate

- SkillsBench (arXiv 2602.12670; 87 tasks, 18 model–harness configs):
  curated skills lift pass rate 33.9% → 50.5% (+16.6 pp); **self-generated
  skills provide no benefit on average**; focused skills with ≤ 3 modules
  beat exhaustive bundles; 16/84 tasks show negative deltas.
- CODESKILL (arXiv 2605.25430), SkillGLoW (2609.02217), GSE (2608.06153):
  gains come from trajectory distillation **plus** replay-driven verification
  and a commit gate that admits a skill only when execution shows no
  degradation of the deployed library.
- Ciel gap: `REGRESSION_DETECTION.md` is prose; no paired with/without
  replay exists before Council review of a skill mutation.

### 3.4 Retrieval at scale — dependency-aware skill graphs

- Graph-of-Skills (arXiv 2604.05333, EMNLP '26): offline skill graph from
  SKILL.md packages; hybrid semantic–lexical seeding; *reverse-aware*
  Personalized PageRank; context-budgeted hydration. On SkillsBench:
  +25.6% peak reward, −56.7% tokens vs. loading all skills (GPT-5.2 Codex);
  consistent across 200–2,000 skill libraries. Reverse traversal matters more
  than graph diffusion itself.
- SE-GoS (2609.08228) evolves the graph from execution traces without
  retraining (52.4% → 59.4% reward, −⅓ tokens).
- Ciel position: 200 installed skills; flat trigger registry; per-skill
  `dependencies.skills` already declared but unused for retrieval.

### 3.5 Memory — hooks should feed recall

- Harness-plugin memories dominate adoption: claude-mem (~91k★), agentmemory
  (95.2% R@5 LongMemEval-S, local-first). Their edge is auto-capture via
  hooks on every prompt/tool/stop event, injected at resume.
- MemPalace reports ~96.6% LongMemEval (self-reported) but is characterized
  by peers as lacking an integration surface (no hooks, no MCP).
- Ciel gap: hooks write `activity.log` only; nothing flows into MemPalace.

### 3.6 Policy-as-code for tool gating

- Vercel `@ai-sdk/policy-opa` (Rego, `allow / deny / requires-approval`),
  AWS Cedar HITL gates (`@tier("hard")` non-overridable vs `@tier("soft")`
  ask-a-human), both testable in CI as reviewable artifacts.
- `sentinel-hooks/claude-code-guardrails` ships a red-team harness that
  fires real PreToolUse JSON events at *any* hook and asserts exit codes.
- Ciel gap: risk regexes are duplicated literals in two hook scripts; parity
  drift was found and fixed on 2026-09-23.

### 3.7 Governance — model diversity in the Council

- llm-council data: same-family judges favor their own style; heterogeneity
  is what catches correlated hallucinations. Ciel's five lenses run on one
  model per session.

### 3.8 Observability

- OpenTelemetry GenAI semantic conventions now define `invoke_agent`,
  `execute_tool`, retrieval and memory spans with `gen_ai.*` attributes;
  reference conformance exists for CrewAI, LangChain, OpenAI Agents, etc.
- Ciel's `observability/OTEL.md` predates this; `activity.log` is bespoke.

## 4. Decisions

See `architecture/ADR_20260923_SKILL_SPEC_CONFORMANCE_AND_SAFETY_GATES.md`.
Workstreams approved 2026-09-23: (1) spec-conformant skills via `ciel.yaml`
sidecar; (2) static skill scan gate; (3) paired-eval commit gate for skill
mutations; (6) policy-as-code hook rules with a red-team harness.
Deferred: (4) skill graph retrieval, (5) hook→memory ingestion,
(7) model-diverse Council, (8) OTel span mapping.

## 5. Watch items

- Re-run `skills-ref validate` across all skills in CI; drift here is silent.
- Track skillfrisk rule updates; a scan gate is only as current as its rules.
- Revisit SkillsBench methodology when building the paired-eval harness so
  Ciel's numbers are comparable to published ones.
