import http from 'node:http';
import https from 'node:https';

const apiUrl = process.env.OBSIDIAN_API_URL || 'http://127.0.0.1:27123';
const apiKey = process.env.OBSIDIAN_API_KEY || '';

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

const today = new Date().toISOString().split('T')[0];
const content = `---
title: "${today}: Initialize Ciel brain for the Ciel project"
date: ${today}
session_id: run-init-brain-1
project: ciel
tags: [diary, session, obsidian, brain-init]
status: completed
---

# ${today}: Initialize Ciel brain for the Ciel project

## Summary

Initialized the Obsidian brain for the Ciel project itself. Four read-only subagents gathered context in parallel, then Ciel synthesized and wrote the project overview, created missing vault folders, and re-indexed hybrid search.

## Actions

- Dispatched 4 subagents to gather: (1) repo structure & verification commands, (2) existing vault state, (3) skills inventory, (4) Obsidian backend readiness.
- Created missing vault folders: \`raw/\`, \`wiki/\`, and \`ciel/projects/ciel/\`.
- Wrote \`ciel/projects/ciel/ciel.md\` with project state, layout, decisions, open tensions, and verification commands.
- Updated \`ciel/projects.md\` index with a Dataview projects table.
- Re-indexed the vault with \`obsidian-hybrid-search reindex\` (18 files).
- Ran \`node ciel.skill/memory/backends/obsidian/cli.mjs --self-test\` — all checks passed.

## Decisions

- Keep the default vault as the single shared \`obsidian-brain/\`; per-project workspaces live under \`ciel/projects/<project>/\`.

## Next Steps

1. Backfill \`raw/\` and \`wiki/\` with source material and synthesized knowledge.
2. Address the six Council mitigations from the migration audit.
3. Migrate any old \`.mempalace/\` partition data if needed.
`;

await request(`/vault/${encodeURIComponent(`ciel/diary/${today}-init-ciel-brain.md`)}`, 'PUT', content);
console.log(`Wrote ciel/diary/${today}-init-ciel-brain.md`);
