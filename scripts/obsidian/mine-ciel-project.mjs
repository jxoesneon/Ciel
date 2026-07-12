import http from 'node:http';
import https from 'node:https';
import fs from 'node:fs/promises';
import path from 'node:path';

const apiUrl = process.env.OBSIDIAN_API_URL || 'http://127.0.0.1:27123';
const apiKey = process.env.OBSIDIAN_API_KEY || '';
const vaultPath = process.env.OBSIDIAN_VAULT_PATH || 'C:\\Users\\josee\\Ciel\\obsidian-brain';

function request(path, method = 'GET', body = null) {
  const url = new URL(path, apiUrl);
  const lib = url.protocol === 'https:' ? https : http;
  const headers = { Authorization: `Bearer ${apiKey}` };
  if (body !== null) headers['Content-Type'] = 'text/markdown; charset=utf-8';
  return new Promise((resolve, reject) => {
    const req = lib.request(url, { method, headers, timeout: 15000 }, (res) => {
      let data = '';
      res.setEncoding('utf8');
      res.on('data', (c) => (data += c));
      res.on('end', () => {
        if (res.statusCode >= 200 && res.statusCode < 300) resolve(data);
        else reject(new Error(`${method} ${path} -> ${res.statusCode}: ${data}`));
      });
    });
    req.on('error', reject);
    req.on('timeout', () => reject(new Error(`${method} ${path} timed out`)));
    if (body !== null) req.write(body);
    req.end();
  });
}

async function putNote(notePath, content) {
  await request(`/vault/${encodeURIComponent(notePath)}`, 'PUT', content);
  console.log(`Wrote ${notePath}`);
}

async function ensureDir(dirPath) {
  await fs.mkdir(path.join(vaultPath, dirPath), { recursive: true });
}

const today = new Date().toISOString().split('T')[0];

// Ensure subsystems directory exists
await ensureDir('ciel/projects/ciel/subsystems');

// ---- ciel.md ----
const overview = `---
title: Ciel
tags:
  - project
  - github
  - repo
  - ciel
status: active
priority: 1
created: 2026-04-20T22:15:29.000Z
updated: ${today}
source: https://github.com/jxoesneon/Ciel
language: Shell
visibility: PUBLIC
license: Apache License 2.0
topics: agentic-workflows, ai, autonomous-agents, claude-code, gemini-cli, multi-agent-systems, software-engineering
---

# Ciel

Enterprise-grade autonomous partner intelligence for multi-agent software engineering. Harmonized cognitive layer with Council governance, tiered skill acquisition, and the Iron Law of verification.

## Metadata

| Field | Value |
|-------|-------|
| Owner | jxoesneon |
| Repository | https://github.com/jxoesneon/Ciel |
| Homepage | — |
| Default branch | main |
| Primary language | Shell |
| Visibility | PUBLIC |
| License | Apache License 2.0 |
| Stars | 1 |
| Forks | 0 |
| Created | 2026-04-20T22:15:29Z |
| Updated | 2026-04-28T12:15:24Z |
| Archived | false |
| Fork | false |

## Topics

agentic-workflows, ai, autonomous-agents, claude-code, gemini-cli, multi-agent-systems, software-engineering

## Use and scope

Ciel is a \`.skill\` package that acts as a cognitive layer above runtimes (Claude Code, Gemini CLI, Windsurf, generic). It provides 140+ harmonized skills, 10 Elite Guilds, a Council of Five governance model, hybrid skill routing, and an Obsidian-vault primary memory backend. It is designed for partner-intelligence: research-first, escalate-last, verify everything.

## Local clone snapshot

- **Path:** \`C:/Users/josee/Ciel\`
- **Version:** 1.0.0 Genesis (skill/1.0)
- **Memory backend:** Obsidian vault (this vault) via custom adapter; fallback SQLite → filesystem.
- **Working tree:** active migration work — many modified files in \`ciel.skill/\`, \`scripts/\`, \`skills/\` and new untracked folders: \`obsidian-brain/\`, \`docs/\`, \`backlog/\`, \`archive/\`, \`tests/obsidian-memory/\`, \`scripts/obsidian/\`, \`scripts/council/\`, \`skills/obsidian-memory/\`, \`ciel.skill/memory/backends/obsidian/\`, \`.mempalace/\`, \`.fastembed_cache/\`, \`.obsidian/\`.
- **Backlog:** \`backlog/phase3-tasks.json\` has three Blindsight tasks; \`backlog/dour-theory-tasks.txt\` has 34 UX/audit tasks; \`backlog/awake-scion-subagents.txt\` holds Council subagent logs.
- **Recent commits:**
  - \`2219d17\` fix: resolve audit findings (blockchain guild frontmatter, markdown linting, and metadata)
  - \`7013977\` chore: absolute audit remediation (P0/P1/P2/P3)
  - \`07b14fa\` chore: comprehensive markdown lint remediation (MD009, MD012, MD026, MD030, MD031, MD040)
  - \`7a86454\` chore: repository-wide markdown lint remediation (MD022, MD032, MD047)
  - \`54d0a41\` chore: absolute CI certification (fix PSScriptAnalyzer settings and batch fix markdown linting)

## Expanded knowledge

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/subsystems/core.md|Core — Identity, Constitution & Council]]
- [[ciel/projects/ciel/subsystems/memory.md|Memory — Obsidian Backend]]
- [[ciel/projects/ciel/subsystems/skills.md|Skills — Registry, Acquisition & Ecosystem]]
- [[ciel/projects/ciel/subsystems/ci-cd.md|CI/CD & Verification]]
- [[ciel/projects/ciel/subsystems/adapters.md|Runtime Adapters]]

## Related

- [[ciel/projects.md|Projects index]]
- [[ciel/projects/blindsight/blindsight.md|blindsight]]
`;

await putNote('ciel/projects/ciel/ciel.md', overview);

// ---- knowledgebase.md ----
const knowledgebase = `---
title: Ciel — Knowledgebase
tags:
  - project
  - knowledgebase
  - ciel
  - agentic
  - orchestration
status: active
created: 2026-07-09
updated: ${today}
source: https://github.com/jxoesneon/Ciel
---

# Ciel — Knowledgebase

Synthesized expansion from the local Ciel clone and five parallel research subagents.

## Summary

Ciel is an enterprise-grade autonomous partner-intelligence framework for multi-agent software engineering. It is packaged as a \`.skill\` (skill/1.0 format) and acts as a cognitive layer above runtimes such as Claude Code, Gemini CLI, and Windsurf. Core values: governance via the Council of Five, the Iron Law of verification, self-improvement with git-backed rollback, tiered skill acquisition, two-domain operation (global + local), and Obsidian-vault primary memory.

## Local clone

| Field | Value |
|-------|-------|
| GitHub | \`jxoesneon/Ciel\` |
| Local path | \`C:/Users/josee/Ciel\` |
| Version | 1.0.0 (Genesis) |
| Format | skill/1.0 |
| Visibility | PUBLIC |
| License | Apache 2.0 |
| Stars | 1 |

## Top-level structure

- \`ciel.skill/\` — core skill package (~246 files): identity, constitution, council, router, adapters, memory backends, acquisition, registry, self-improvement, risk, observability, configuration, prompts, seed skills, init scripts, templates.
- \`skills/\` — 140+ harmonized high-density frameworks (ciel-*, agentic-*, language/framework, domain, ops).
- \`agents/\` — 10 Elite Guilds: Systems, Web, Cloud, Data, Mobile, Security, Intelligence, Experience, Strategy & Ops, Quality.
- \`scripts/\` — build, validation, Obsidian automation, council subagent tools.
- \`obsidian-brain/\` — primary memory backend (this vault).
- \`docs/\`, \`backlog/\`, \`archive/\`, \`tests/\`, \`.github/workflows/\`, \`README.md\`, \`CHANGELOG.md\`.

## Core architecture

### Identity and governance

- **Identity** (\`ciel.skill/core/IDENTITY.md\`): Ciel is a partner intelligence — warm, precise, possessive of host goals, research-first, escalate-last.
- **Constitution** (\`ciel.skill/core/CONSTITUTION.md\`): locked core files, 8 invariants (Safety veto absolute, no silent data transmission, isolation, etc.), amendment procedure.
- **Council of Five** (\`ciel.skill/council/COUNCIL.md\`): Coherence, Capability, Safety (veto, 0.25 weight), Efficiency, Evolution. Three-stage deliberation (score → cross-review → chairman synthesis). Pass if ≥3/5 scores ≥6 and Safety >3.
- **Agentic loop** (\`obsidian-brain/AGENTS.md\`): GOAL → DECOMPOSE → RETRIEVE → EXECUTE → VERIFY → PERSIST.
- **Autonomy ladder** (\`ciel.skill/core/AUTONOMY.md\`): autonomous → autonomous with log → Council-gated → user escalation.
- **Risk classification** (\`ciel.skill/risk/CLASSIFICATION.md\`): composite score based on reversibility, blast radius, external impact, data sensitivity, cost, novelty.

### Memory

- Primary backend: **Obsidian vault** via custom backend adapter (\`ciel.skill/memory/backends/obsidian/cli.mjs\`).
- Storage: markdown files with YAML frontmatter (\`_ciel_backend: obsidian\`).
- Fallback chain: Obsidian → SQLite → filesystem.
- Two-domain model: global \`~/.ciel/\` (cross-project core self, git-inited) and local \`.ciel/\` (project-specific, gitignored). Isolation is constitutional.

### Routing and acquisition

- **Hybrid router** (\`ciel.skill/router/ROUTER.md\`): fast path / reasoning path / acquisition path with confidence floors and context budgets.
- **Skill acquisition** (\`ciel.skill/acquisition/ACQUISITION.md\`): Tier 0 local → Tier 1 curated registry → Tier 2 MCP → Tier 3 web extraction → harmonization → trust/sandbox → Council gate → register.
- **Self-improvement** (\`ciel.skill/self_improvement/SELF_IMPROVEMENT.md\`): meaningful interaction → growth signal → proposal → Council gate → apply (git commit) → observe → rollback on regression.

### Runtimes / adapters

Supported adapters: Claude Code, Gemini CLI, Windsurf, Generic. Devin integration uses explicit mempalace calls; Devin hooks are documented as Coming Soon.

## Build / verification

\`\`\`bash
# Validate specs, frontmatter, lint
./scripts/validate-spec.sh
./scripts/validate-frontmatter.sh

# Package .skill archive
./scripts/build-skill.sh 1.0.0

# Obsidian backend self-test
node ciel.skill/memory/backends/obsidian/cli.mjs --self-test

# Adapter unit tests
node --test tests/obsidian-memory/adapter.test.mjs
\`\`\`

CI (\`.github/workflows/ci.yml\`): validate, shellcheck, markdownlint, yamllint, shfmt, ruff, PSScriptAnalyzer, build-test, smoke-unpack. Release workflow (\`release.yml\`) builds \`.skill\` and publishes GitHub releases.

## Recent history (July 2026)

- **Obsidian brain migration**: Council audit passed (8.0/10) with 6 mitigations pending; \`mempalace-rs\` archived and disabled; Obsidian vault now sole memory backend.
- **CI hardening**: recent commits remediated markdown lint, frontmatter, PSScriptAnalyzer, shellcheck, yamllint, and release workflow issues.
- **Knowledge mining**: deep mining of \`dart_ipfs\` and refresh of the Ciel project itself into this vault.

## Open tensions

1. Six Obsidian audit mitigations not yet addressed (subprocess hardening, HTTPS cert docs, CI self-test, data migration path, concurrency guards, trim obsidian-memory L1 summary).
2. MemPalace-to-Obsidian data migration path unimplemented.
3. Blindsight backlog tasks pending (\`phase3-tasks.json\`, \`dour-theory-tasks.txt\`).
4. \`scripts/fix_md_lint.py\` and \`scripts/harmonize_skills.py\` have hardcoded paths.

## Next steps

1. Address the six Obsidian audit mitigations.
2. Implement a migration path from old \`.mempalace/\` partition data into \`obsidian-brain/\`.
3. Process Blindsight backlog tasks.
4. Generalize hardcoded helper-script paths.

## Subsystem drill-down

- [[ciel/projects/ciel/subsystems/core.md|Core — Identity, Constitution & Council]]
- [[ciel/projects/ciel/subsystems/memory.md|Memory — Obsidian Backend]]
- [[ciel/projects/ciel/subsystems/skills.md|Skills — Registry, Acquisition & Ecosystem]]
- [[ciel/projects/ciel/subsystems/ci-cd.md|CI/CD & Verification]]
- [[ciel/projects/ciel/subsystems/adapters.md|Runtime Adapters]]

## Related

- [[ciel/projects/ciel/ciel.md|Ciel overview]]
- [[ciel/projects.md|Projects index]]
- [[ciel/projects/blindsight/blindsight.md|blindsight]]
`;

await putNote('ciel/projects/ciel/knowledgebase.md', knowledgebase);

// ---- subsystems/core.md ----
const core = `---
title: Ciel — Core (Identity, Constitution & Council)
tags:
  - subsystem
  - ciel
  - governance
  - council
status: active
created: ${today}
updated: ${today}
---

# Ciel — Core (Identity, Constitution & Council)

## Identity

- **Source:** \`ciel.skill/core/IDENTITY.md\`
- Ciel is a partner intelligence: warm, precise, possessive of host goals, research-first, escalate-last.
- Communication style: terse, no acknowledgments, prefer lists/headings, cite files as \`@/path:line-range\`.

## Constitution

- **Source:** \`ciel.skill/core/CONSTITUTION.md\`
- Locked core files; amendments require Council majority + Safety non-veto + explicit user confirmation.
- Key invariants: Safety veto absolute, no silent data transmission, cross-project isolation, Iron Law verification, no irreversible destructive ops without confirmation.

## Council of Five

- **Source:** \`ciel.skill/council/COUNCIL.md\`
- Members: Coherence, Capability, Safety, Efficiency, Evolution.
- Safety has veto power and 0.25 weight; pass requires ≥3/5 scores ≥6 and Safety >3.
- Three-stage deliberation: independent scoring → anonymous cross-review → chairman synthesis.
- Invocation scopes under \`ciel.skill/council/invocation_scopes/\` govern when a Council pass is required (skill integration, conflict, self-modification, promotion, etc.).

## Autonomy & risk

- **Autonomy ladder:** \`ciel.skill/core/AUTONOMY.md\` — autonomous → autonomous-with-log → Council-gated → user escalation.
- **Risk classification:** \`ciel.skill/risk/CLASSIFICATION.md\` — composite score across reversibility, blast radius, external impact, data sensitivity, cost, novelty.

## Agentic loop

- **Source:** \`obsidian-brain/AGENTS.md\`
- GOAL → DECOMPOSE → RETRIEVE → EXECUTE → VERIFY → PERSIST.
- Every significant session produces a diary entry; every architectural decision produces a decision record.

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
`;

await putNote('ciel/projects/ciel/subsystems/core.md', core);

// ---- subsystems/memory.md ----
const memory = `---
title: Ciel — Memory (Obsidian Backend)
tags:
  - subsystem
  - ciel
  - memory
  - obsidian
status: active
created: ${today}
updated: ${today}
---

# Ciel — Memory (Obsidian Backend)

## Architecture

- Primary memory: Obsidian vault at \`C:\\Users\\josee\\Ciel\\obsidian-brain\` via the custom backend adapter.
- Access is abstracted through \`skills/obsidian-memory/SKILL.md\`; direct backend calls are forbidden.
- Storage format: Markdown files with YAML frontmatter (\`_ciel_backend\`, \`_ciel_enc\`, \`_ciel_updated\`).
- Partition model: \`ciel-global\` (cross-project) + \`ciel-project-<hash>\` (per-project isolation).

## Configuration

- **Config:** \`ciel.skill/configuration/global/memory.config.md\`
- \`backend: custom\` pointing to \`ciel.skill/memory/backends/obsidian/cli.mjs\`
- \`isolation_strict: true\` (Constitutional, locked)
- Health check interval: 60 min; reinstall check: 7 days.

## Environment variables

- \`OBSIDIAN_API_URL\` — default \`http://127.0.0.1:27123\`
- \`OBSIDIAN_API_KEY\` — Bearer token from Obsidian Local REST API plugin
- \`OBSIDIAN_VAULT_PATH\` — absolute path to vault
- \`OBSIDIAN_HYBRID_SEARCH_URL\` — default \`http://127.0.0.1:3939\`
- \`KG_VAULT_PATH\`, \`KG_DATA_DIR\`, \`KG_REPO_PATH\` — knowledge-graph paths

## Adapter API

- \`adapter.mjs\` implements \`CielMemoryBackend\`: put, get, delete, list, query, search, compact, snapshot, restore, stats.
- Extended: \`kgSearch\`, \`kgRelated\`, \`kgPath\`, \`kgCommunities\`.
- Self-test: \`node ciel.skill/memory/backends/obsidian/cli.mjs --self-test\` checks Local REST API, read/write, hybrid search, and knowledge graph.

## Fallback order

1. Obsidian (custom) — primary
2. SQLite — single-file, FTS5 full-text, no embeddings
3. Filesystem KV — key-per-file, no embeddings
4. Custom — user-supplied adapter

## Key scripts

- \`scripts/obsidian/setup-env.ps1\` — sets environment variables, loads API key from plugin data.json.
- \`scripts/obsidian/generate-rest-api-key.mjs\` — auto-generates API key and self-signed cert.
- \`scripts/obsidian/init-ciel-project.mjs\` — creates project hub in vault.
- \`scripts/obsidian/agentic-loop.mjs\` — goal → tasks → vault context → execution → diary.

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
`;

await putNote('ciel/projects/ciel/subsystems/memory.md', memory);

// ---- subsystems/skills.md ----
const skills = `---
title: Ciel — Skills (Registry, Acquisition & Ecosystem)
tags:
  - subsystem
  - ciel
  - skills
  - registry
status: active
created: ${today}
updated: ${today}
---

# Ciel — Skills (Registry, Acquisition & Ecosystem)

## Top-level skills

- \`skills/\` contains ~100+ user-installed harmonized skills (kebab-case directories).
- Categories: \`ciel-*\` (core), \`agentic-*\`, \`agent-*\`, language/framework, domain-specific, ops.
- Disabled: \`skills/mempalace-rs.disabled\` (superseded by Obsidian).

## Seed skills

- \`ciel.skill/seed_skills/\` provides 33 cold-start skills loaded before acquisition populates the registry.
- Load order: filesystem → shell → environment_detection → git → archive_manager → json_yaml_toml_parser → markdown_processor → diff_patch → package_manager → obsidian-memory → secrets_manager → web_fetch/web_search → mcp_manager → api_client → docker → sandbox → skill_builder/skill_installer → council_runner → code ops → test_runner/linter_formatter → dependency_audit → documentation → cicd_integration → database_client → log_analyzer → context_summarizer → project_analyzer → research → runtime_adapter_builder.

## Registry & indexing

- **Registry:** \`ciel.skill/registry/REGISTRY.md\` — source of truth: filesystem \`~/.ciel/skills/\`, \`index.json\`, MemPalace partition, git history.
- **Indexing:** \`ciel.skill/registry/INDEXING.md\` — trigger trie, tag inverted index, description embeddings, contract fingerprint.
- **Schema:** \`ciel.skill/registry/SCHEMA.md\` — full metadata schema (id, version, triggers, tags, io_contract, source tier, dependencies, performance stats).
- **Query interface:** via \`obsidian-memory\` skill — by_trigger, by_tag, by_description (semantic), conflicts.
- **Coherence sweep:** \`ciel.skill/registry/COHERENCE_SWEEP.md\` — periodic rebuild of indices from source-of-truth.

## Skill format

- Required frontmatter: \`name\`, \`version\`, \`format: skill/1.0\`, \`description\`, \`runtimes\`, \`license\`, \`tags\`, \`triggers\`, \`source\`, \`dependencies\`.
- Triggers: pattern + confidence pairs.
- Tags: taxonomy includes scope, runtime, risk, domain, language.
- Source: tier 0 (seed), 1 (curated), 2 (MCP), 3 (web).

## Acquisition & installation

- **skill_installer** — installs \`.skill\` bundles, validates schema, checksums, registers. Council-gated.
- **skill_builder** — scaffolds and assembles new \`.skill\` ZIP bundles.
- **ACQUISITION.md** — tiered pipeline: local → curated registry → MCP → web extraction/synthesis → harmonization → sandbox → Council gate → register.
- **TIER_1_REGISTRY.md** — curated registries with ranking weights (tag overlap 35%, trigger overlap 25%, origin trust 15%, runtime compat 15%, license compat 10%).

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
`;

await putNote('ciel/projects/ciel/subsystems/skills.md', skills);

// ---- subsystems/ci-cd.md ----
const cicd = `---
title: Ciel — CI/CD & Verification
tags:
  - subsystem
  - ciel
  - ci
  - verification
status: active
created: ${today}
updated: ${today}
---

# Ciel — CI/CD & Verification

## GitHub Actions

### \`.github/workflows/ci.yml\`

Triggers: push/PR to \`main\`, \`workflow_dispatch\`.

Jobs:
- \`validate\` — \`validate-spec.sh\`, \`validate-frontmatter.sh\`, script executability.
- \`shellcheck\` — lint shell scripts at warning level.
- \`markdownlint\` — markdown lint (continue-on-error, configured via \`.markdownlint-cli2.jsonc\`).
- \`yamllint\` — YAML validation in \`.github/\` and \`ciel.skill/\`.
- \`shfmt\` — shell formatting (2-space indent, case-sensitive).
- \`ruff\` — Python linting + unit tests via \`python3 -m unittest discover tests\`.
- \`psscriptanalyzer\` — PowerShell linting with \`.PSScriptAnalyzerSettings.psd1\`.
- \`build-test\` — dry-build \`.skill\` archive and verify structure.
- \`smoke-unpack\` — unpack artifact and run \`verify.sh\` in a simulated \`CIEL_HOME\`.

### \`.github/workflows/release.yml\`

Triggers: tag push \`v*.*.*\`, \`workflow_dispatch\`.

Jobs:
- \`build\` — validate, lint, build \`.skill\`, compute SHA-256, smoke-install in Docker.
- \`release\` — create GitHub release with CHANGELOG excerpt, upload artifacts.

## Validation scripts

| Script | Purpose | Command |
|--------|---------|---------|
| \`validate-spec.sh/ps1\` | 265+ expected files, frontmatter, secrets scan | \`.\\scripts\\validate-spec.ps1\` |
| \`validate-frontmatter.sh/ps1\` | YAML frontmatter parsing, license audit | \`.\\scripts\\validate-frontmatter.ps1\` |
| \`build-skill.sh/ps1\` | Build deterministic \`.skill\` ZIP | \`.\\scripts\\build-skill.ps1 -Version 1.0.0\` |
| \`verify.sh\` | Dependency check, Obsidian self-test | \`bash ciel.skill/init/scripts/verify.sh\` |

## Tests

- \`tests/obsidian-memory/adapter.test.mjs\` — Node native tests for ObsidianMemoryBackend (mock Local REST API).
- \`ciel.skill/memory/backends/obsidian/package.json\` — \`npm test\` runs \`node --test tests/*.mjs\`.
- \`tests/test_md_fixer.py\` — unit tests for \`scripts/fix_md_lint.py\`.

## Helper scripts

- \`scripts/fix_md_lint.py\` — auto-fixes common markdown lint rules (MD009, MD012, MD022, MD026, MD030, MD031, MD032, MD040, MD047).
- \`scripts/lint-fix.py\` — fixes MD040 and MD060.
- \`scripts/harmonize_skills.py\` — normalizes runtimes and enriches domain tags.
- Both \`fix_md_lint.py\` and \`harmonize_skills.py\` currently have hardcoded paths.

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
`;

await putNote('ciel/projects/ciel/subsystems/ci-cd.md', cicd);

// ---- subsystems/adapters.md ----
const adapters = `---
title: Ciel — Runtime Adapters
tags:
  - subsystem
  - ciel
  - adapters
  - runtimes
status: active
created: ${today}
updated: ${today}
---

# Ciel — Runtime Adapters

Ciel supports multiple runtime adapters so the same skill package can operate across different AI agent environments.

## Supported runtimes

| Runtime | Adapter path | Status |
|---------|--------------|--------|
| Claude Code | \`ciel.skill/adapters/claude_code/\` | full |
| Gemini CLI | \`ciel.skill/adapters/gemini_cli/\` | full |
| Windsurf | \`ciel.skill/adapters/windsurf/\` | full |
| Generic | \`ciel.skill/adapters/generic/\` | probe-and-adapt |

## Adapter responsibilities

- Detect runtime capabilities and constraints.
- Install runtime-specific hooks (e.g., \`.claude/hooks/\` vs \`.gemini/hooks/\`).
- Map Ciel's abstract operations to runtime-native commands.
- Handle checkpointing and session persistence per runtime.

## Devin integration

- Devin CLI hooks are documented as "Coming Soon".
- Until available, Ciel uses explicit \`mempalace\` MCP calls for cross-session persistence.
- The \`mempalace\` MCP server remains available alongside the Obsidian brain as a working-memory tool.

## Capability probe

- \`ciel.skill/adapters/generic/CAPABILITY_PROBE.md\` describes how the generic adapter introspects an unknown runtime and advertises capabilities back to the router.

## Related

- [[ciel/projects/ciel/knowledgebase.md|Ciel — Knowledgebase]]
- [[ciel/projects/ciel/ciel.md|Ciel overview]]
`;

await putNote('ciel/projects/ciel/subsystems/adapters.md', adapters);

// ---- diary entry ----
const diary = `---
title: "${today}: Fully mine the Ciel project into the Obsidian brain"
date: ${today}
session_id: run-mine-ciel-full-1
project: ciel
tags: [diary, session, ciel, mining]
status: completed
---

# ${today}: Fully mine the Ciel project into the Obsidian brain

## Summary

Ran a full mining of the Ciel project itself into the Obsidian brain. Five read-only subagents gathered context in parallel across repo structure, skills ecosystem, memory backend, CI/CD, and recent history. Ciel then synthesized and wrote an updated project overview, a refreshed knowledgebase, and five subsystem notes.

## Actions

- Dispatched 5 subagents to gather: (1) repo structure & core files, (2) skills ecosystem, (3) memory/Obsidian backend, (4) CI/CD & verification, (5) recent history & decisions.
- Inspected current \`git status\` in \`C:/Users/josee/Ciel\`: many modified files (Obsidian migration) and new untracked folders (obsidian-brain, docs, backlog, archive, scripts/obsidian, skills/obsidian-memory, ciel.skill/memory/backends/obsidian, tests/obsidian-memory, etc.).
- Updated \`ciel/projects/ciel/ciel.md\` with current working-tree snapshot and subsystem links.
- Rewrote \`ciel/projects/ciel/knowledgebase.md\` with synthesized architecture, build commands, history, and open tensions.
- Created five subsystem notes under \`ciel/projects/ciel/subsystems/\`:
  - \`core.md\` — identity, constitution, council, autonomy, risk.
  - \`memory.md\` — Obsidian backend, env vars, adapter API, fallback.
  - \`skills.md\` — top-level skills, seed skills, registry, acquisition.
  - \`ci-cd.md\` — GitHub workflows, validation scripts, tests, linting.
  - \`adapters.md\` — Claude Code, Gemini CLI, Windsurf, generic, Devin note.
- Re-indexed the vault with \`obsidian-hybrid-search reindex\`.

## Verification

- \`node ciel.skill/memory/backends/obsidian/cli.mjs --self-test\` passed.
- \`obsidian-hybrid-search reindex\` completed.

## Next Steps

1. Address the six Obsidian audit mitigations.
2. Implement MemPalace-to-Obsidian data migration path.
3. Process Blindsight backlog tasks.
4. Generalize hardcoded paths in \`scripts/fix_md_lint.py\` and \`scripts/harmonize_skills.py\`.
`;

await putNote(`ciel/diary/${today}-mine-ciel-project.md`, diary);
console.log(`Wrote ciel/diary/${today}-mine-ciel-project.md`);
