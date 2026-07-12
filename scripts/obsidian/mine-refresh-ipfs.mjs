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

async function deleteNote(path) {
  try {
    await request(`/vault/${encodeURIComponent(path)}`, 'DELETE');
    console.log(`Deleted ${path}`);
  } catch (e) {
    if (e.message.includes('404')) console.log(`Already missing: ${path}`);
    else console.error(`Failed to delete ${path}: ${e.message}`);
  }
}

async function updateNote(path, replacer) {
  const current = await request(`/vault/${encodeURIComponent(path)}`);
  const updated = replacer(current);
  await request(`/vault/${encodeURIComponent(path)}`, 'PUT', updated);
  console.log(`Updated ${path}`);
}

await deleteNote('ciel/projects/IPFS/tasks/plan-run-mrfzkrup.md');
await deleteNote('ciel/projects/IPFS/tasks/task-t1-run-mrfzkrup.md');
await deleteNote('ciel/projects/IPFS/tasks/task-t2-run-mrfzkrup.md');
await deleteNote('ciel/projects/IPFS/tasks/task-t3-run-mrfzkrup.md');

const today = new Date().toISOString().split('T')[0];

await updateNote('ciel/projects/IPFS/IPFS.md', (content) =>
  content
    .replace(/- Version: v1\.11\.\d+ \(released [^)]+\)/, '- Version: v1.11.7 (released 2026-07-11)')
    .replace(/- Tests: [\d,]+ passing, \d+ skipped, .*?failing.*\n/, '- Tests: 3478 passing, 8 skipped, 0 failing on Windows VM; interop tests requiring Kubo/Helia are run separately in CI.\n')
    .replace(/- Working tree: extensive uncommitted changes and many untracked files; active stabilization toward next release\./, '- Working tree: clean (no uncommitted changes).')
    .replace(/updated: 2026-07-11/, `updated: ${today}`)
);

await updateNote('ciel/projects/IPFS/git-state.md', (content) =>
  content
    .replace(/- \*\*Version:\*\* \`dart_ipfs\` \*\*v1\.11\.\d+\*\* \(released [^)]+\)\./, '- **Version:** `dart_ipfs` **v1.11.7** (released 2026-07-11).')
    .replace(/- \*\*Tag:\*\* \`v1\.11\.\d+\`\./, '- **Tag:** `v1.11.7`.')
    .replace(/updated: 2026-07-09/, `updated: ${today}`)
    .replace(/## Recent commits\n\nFrom local \`git log\`:\n\n\| Hash \| Message \|\n\|------\|---------\|\n[\s\S]*?\n## Working tree status/, () => {
      const recent = `| Hash | Message |
|------|---------|
| \`23aeeb07\` | chore: remove temporary build log |
| \`ee22ec91\` | fix(pubspec): remove direct xml/dart_udx deps that break downstream consumers |
| \`4c44553b\` | ci(docker): skip SBOM release asset upload to avoid permission failure |
| \`687c8472\` | chore(release): bump version to 1.11.6 and clean publishing artifacts |
| \`3c22338b\` | chore: remove unnecessary dart_style dev dependency |
| \`6991c7a6\` | fix(gateway): include leading slash in HTTPS redirect location |
| \`3adbb882\` | ci: use expanded test reporter to capture failing test name (temporary) |
| \`b3fb689e\` | fix(test): apply dart_style formatting to interop tests and add dart_style dev dependency |
| \`dfd79667\` | ci: show formatting diff on Ubuntu (temporary) |
| \`a723e244\` | fix(test): run IPFSWebNode tests offline and fix interop/gateway CI assertions |
| \`65a45457\` | fix(web,test): make dart_ipfs web-compatible and harden CI tests |
| \`c6845ccc\` | fix(web): make dart_ipfs Flutter-web compatible |
| \`a50e3d88\` | chore(deps): update GitHub Actions, patch Helia DHT vulnerability, and refresh Dart packages |
| \`f1f5d975\` | refactor: professionalize documentation and remove internal tooling references |
| \`f0cde921\` | fix: resolve Kubo/Helia interop tests and finalize release readiness |`;
      return `## Recent commits\n\nFrom local \`git log\`:\n\n${recent}\n\n## Working tree status`;
    })
    .replace(/## Working tree status \(manual snapshot 2026-07-09\)[\s\S]*?## Notable prior events/, `## Working tree status (${today})\n\nThe local clone is **clean**: no uncommitted changes and no untracked files. The latest tag is \`v1.11.7\` and \`pubspec.yaml\` also reports \`1.11.7\`.\n\n## Notable prior events`)
);

const diaryContent = `---
title: "${today}: Refresh IPFS project mining"
date: ${today}
session_id: run-mine-ipfs-refresh-1
project: IPFS
tags: [diary, session, ipfs, mining]
status: completed
---

# ${today}: Refresh IPFS project mining

## Summary

Re-mined the local \`dart_ipfs\` clone into the Obsidian brain. The agentic loop could not run because the hybrid-search endpoint returned 404, so Ciel performed the mining directly: ran \`dart test\`, inspected git state, and updated the project notes via the Local REST API.

## Actions

- Ran \`dart test\` in \`C:\\\\Users\\\\josee\\\\IPFS\` — 3478 passing, 8 skipped, 0 failing.
- Verified \`git status\` is clean; latest tag is \`v1.11.7\`; \`pubspec.yaml\` version is \`1.11.7\`.
- Removed stale agentic-loop artifacts from \`ciel/projects/IPFS/tasks/\` (plan/task notes for failed run \`run-mrfzkrup\`).
- Updated \`ciel/projects/IPFS/IPFS.md\` with version \`v1.11.7\`, clean working tree, and current test counts.
- Updated \`ciel/projects/IPFS/git-state.md\` with version \`v1.11.7\`, clean status, and recent commit history.
- Re-indexed the vault with \`obsidian-hybrid-search reindex\`.

## Verification

- \`node ciel.skill/memory/backends/obsidian/cli.mjs --self-test\` passed.
- \`dart test\` passed locally.

## Next Steps

1. Continue the README verification goal already in \`ciel/projects/IPFS/goals/readme-verification-update-2026-07-11.md\`.
2. Update subsystem notes if any recent architectural changes affect them.
3. Backfill \`raw/\` with any source material the user wants preserved.
`;

await request(`/vault/${encodeURIComponent(`ciel/diary/${today}-refresh-ipfs-mining.md`)}`, 'PUT', diaryContent);
console.log(`Wrote ciel/diary/${today}-refresh-ipfs-mining.md`);
