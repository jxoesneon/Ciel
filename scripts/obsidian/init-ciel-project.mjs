import http from 'node:http';
import https from 'node:https';

const apiUrl = process.env.OBSIDIAN_API_URL || 'http://127.0.0.1:27123';
const apiKey = process.env.OBSIDIAN_API_KEY || '';

function request(path, method = 'GET', body = null) {
  const url = new URL(path, apiUrl);
  const lib = url.protocol === 'https:' ? https : http;
  const headers = { Authorization: `Bearer ${apiKey}` };
  if (body !== null) {
    headers['Content-Type'] = 'text/markdown; charset=utf-8';
  }
  return new Promise((resolve, reject) => {
    const req = lib.request(url, { method, headers, timeout: 15000 }, (res) => {
      let data = '';
      res.setEncoding('utf8');
      res.on('data', (c) => (data += c));
      res.on('end', () => {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          resolve(data);
        } else {
          reject(new Error(`${method} ${path} -> ${res.statusCode}: ${data}`));
        }
      });
    });
    req.on('error', reject);
    req.on('timeout', () => reject(new Error(`${method} ${path} timed out`)));
    if (body !== null) req.write(body);
    req.end();
  });
}

const today = new Date().toISOString().split('T')[0];

const overview = `---
title: Ciel Project Overview
aliases: [ciel]
tags: [project, ciel, obsidian, status:active]
created: ${today}
updated: ${today}
status: active
priority: 0
---

# Ciel Project Overview

The Ciel orchestration intelligence repository, now operating in full Obsidian mode. This note is the entry point for everything Ciel is doing on this project.

## Current State

- **Branch**: \`Obsidian\`
- **Memory backend**: custom Obsidian vault at \`C:\\Users\\josee\\Ciel\\obsidian-brain\`
- **Backend readiness**: desktop app running, Local REST API enabled, hybrid search running, knowledge graph indexed.
- **Legacy backend**: \`mempalace-rs\` skills archived to \`archive/\` and disabled; they are not deleted.
- **Verification**: Obsidian self-test passes; \`tests/obsidian-memory/adapter.test.mjs\` passes 6/6.

## Repository Layout

| Path | Purpose |
| --- | --- |
| \`skills/\` | 140+ harmonized Ciel skills (orchestration, languages, frameworks, ops) |
| \`ciel.skill/\` | Core infrastructure: constitution, council, memory, registry, adapters (~246 files) |
| \`agents/\` | 10 Elite Guild skill packs |
| \`scripts/\` | Build/validation automation and Obsidian service helpers |
| \`docs/\` | Migration and architecture documentation |
| \`tests/\` | Verification suites |
| \`obsidian-brain/\` | The Obsidian vault that serves as Ciel's persistent brain |
| \`.ciel/\` | Local runtime state and project.json |
| \`archive/\` | Backed-up/disabled legacy skills |

## Key Decisions

- [[ciel/kg/decisions/obsidian-brain-migration-audit]] — Council-approved migration to Obsidian backend (score 8/10, six mitigations pending).
- Generic \`mempalace\` invocations now resolve to the Obsidian backend implementation.
- Per-project data lives under \`ciel/projects/<project>/\ inside the shared vault; the default vault is not per-project.

## Open Tensions / Next Steps

1. Address the six Council mitigations in the migration audit (subprocess hardening, HTTPS cert docs, CI self-test, data migration path, concurrency guards, trim obsidian-memory L1 summary).
2. Create a data migration path from old \`.mempalace/\` partitions into \`obsidian-brain/\`.
3. Update any remaining project-specific skills that hard-code legacy \`mempalace-rs\` commands.
4. Backfill \`raw/\` and \`wiki/\` with source material and synthesized knowledge.
5. Re-run \`scripts/validate-spec.ps1\` after skill tree changes.

## Verification Commands

\`\`\`powershell
# Obsidian backend self-test
node ciel.skill/memory/backends/obsidian/cli.mjs --self-test

# Adapter unit tests
node --test tests/obsidian-memory/adapter.test.mjs

# Spec compliance
.\\scripts\\validate-spec.ps1
\`\`\`

## Related Notes

- [[ciel/identity]] — Ciel's identity and user preferences.
- [[AGENTS]] — Agentic loop protocol.
- [[ciel/kg/concepts/obsidian-skills]] — kepano/obsidian-skills integration.
- [[ciel/diary]] — Session history.
`;

const projectsIndex = `---
title: Projects
tags: [index, project]
created: ${today}
updated: ${today}
---

# Projects

Projects are per-project workspaces under \`ciel/projects/<project>/<project>.md\`.

## Active Projects

\`\`\`dataview
TABLE status, priority, updated
FROM "ciel/projects"
SORT priority ASC, updated DESC
\`\`\`
`;

await request(`/vault/${encodeURIComponent('ciel/projects/ciel/ciel.md')}`, 'PUT', overview);
console.log('Wrote ciel/projects/ciel/ciel.md');

await request(`/vault/${encodeURIComponent('ciel/projects.md')}`, 'PUT', projectsIndex);
console.log('Wrote ciel/projects.md');
