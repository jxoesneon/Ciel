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

async function updateNote(filePath, replacer) {
  const encoded = encodeURIComponent(filePath);
  const current = await request(`/vault/${encoded}`);
  const updated = replacer(current);
  await request(`/vault/${encoded}`, 'PUT', updated);
  console.log(`Updated ${filePath}`);
}

await updateNote('ciel/diary/2026-07-08-integrate-kepano-obsidian-skills.md', (content) =>
  content
    .replace('status: active', 'status: completed')
    .replace(
      '## Open Tensions\n\n- The Obsidian Local REST API is still blocked by restricted mode (pending the manual UI toggle from the previous session).',
      '## Verification\n\n- Re-cloned `kepano/obsidian-skills` to `.ciel/obsidian-skills/` at session time.\n- Confirmed `defuddle` is installed in `scripts/obsidian/` and `npx defuddle parse https://obsidian.md --md` returns clean markdown.\n- Ran `node ciel.skill/memory/backends/obsidian/cli.mjs --self-test`: Local REST API, read-write, hybrid-search, and knowledge-graph all OK.\n\n## Open Tensions\n\n- None.'
    )
    .replace('~/.ciel/skills/obsidian-skills/', '.ciel/obsidian-skills/')
);

await updateNote('ciel/kg/concepts/obsidian-skills.md', (content) =>
  content
    .replace('~/.ciel/skills/obsidian-skills/', '.ciel/obsidian-skills/')
    .replace('status: active', 'status: active')
);

await updateNote('_CLAUDE.md', (content) =>
  content.replace('~/.ciel/skills/obsidian-skills/', '.ciel/obsidian-skills/')
);

console.log('Vault notes refreshed.');
