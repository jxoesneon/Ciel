import { describe, it, before, after } from 'node:test';
import assert from 'node:assert';
import http from 'node:http';
import { ObsidianMemoryBackend } from '../../ciel.skill/memory/backends/obsidian/adapter.mjs';

describe('ObsidianMemoryBackend', () => {
  let server;
  let port;
  let backend;
  const store = new Map();
  const dirs = new Set();

  before(async () => {
    server = http.createServer((req, res) => {
      const url = new URL(req.url, `http://127.0.0.1`);
      const send = (status, body) => {
        res.writeHead(status, { 'Content-Type': 'application/json' });
        res.end(typeof body === 'string' ? body : JSON.stringify(body));
      };

      const pathMatch = url.pathname.match(/^\/vault\/(.*)$/);
      const filePath = pathMatch ? decodeURIComponent(pathMatch[1]) : '';

      if (url.pathname === '/') {
        return send(200, { status: 'OK' });
      }

      if (url.pathname === '/health') {
        return send(200, { ok: true });
      }

      if (url.pathname === '/search') {
        // Mock hybrid search: simple substring match on stored values.
        let body = '';
        req.on('data', d => (body += d));
        req.on('end', () => {
          const params = JSON.parse(body || '{}');
          const raw = params.query || '';
          const folderMatch = raw.match(/folder:([^\s]+)/);
          const scope = folderMatch ? folderMatch[1] : '';
          const q = raw.replace(/folder:\S+/g, '').trim().toLowerCase();
          const results = [];
          for (const [path, content] of store.entries()) {
            if (scope && !path.startsWith(scope + '/')) continue;
            if (content.toLowerCase().includes(q)) {
              results.push({ path, score: 1, snippet: content.slice(0, 100) });
            }
          }
          send(200, results);
        });
        return;
      }

      if (req.method === 'GET' && url.pathname.startsWith('/vault/')) {
        if (store.has(filePath)) {
          return send(200, store.get(filePath));
        }
        if (dirs.has(filePath)) {
          const prefix = filePath ? filePath + '/' : '';
          const entries = [];
          for (const key of store.keys()) {
            if (key.startsWith(prefix)) {
              entries.push(key.slice(prefix.length));
            }
          }
          return send(200, entries);
        }
        return send(404, { error: 'not found' });
      }

      if (req.method === 'PUT' && url.pathname.startsWith('/vault/')) {
        let body = '';
        req.on('data', d => (body += d));
        req.on('end', () => {
          store.set(filePath, body);
          const dir = filePath.includes('/') ? filePath.slice(0, filePath.lastIndexOf('/')) : '';
          dirs.add(dir);
          send(200, { ok: true });
        });
        return;
      }

      if (req.method === 'DELETE' && url.pathname.startsWith('/vault/')) {
        store.delete(filePath);
        return send(200, { ok: true });
      }

      send(404, { error: 'unsupported' });
    });

    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    port = server.address().port;
    backend = new ObsidianMemoryBackend({
      obsidianApiUrl: `http://127.0.0.1:${port}`,
      obsidianApiKey: 'test-key',
      vaultPath: '/tmp/test-vault',
      hybridSearchUrl: `http://127.0.0.1:${port}`,
    });
  });

  after(async () => {
    await new Promise(resolve => server.close(resolve));
  });

  it('round-trips a string value with frontmatter', async () => {
    await backend.put('ciel/projects/ciel', 'overview', Buffer.from('Project context'), {
      title: 'Ciel Overview',
      tags: ['project'],
    });
    const value = await backend.get('ciel/projects/ciel', 'overview');
    assert.strictEqual(value.toString('utf8'), 'Project context');
  });

  it('lists keys in a partition', async () => {
    await backend.put('ciel/projects/ciel', 'overview', Buffer.from('Project context'));
    await backend.put('ciel/projects/ciel', 'requirements', Buffer.from('Requirements'));
    const keys = await backend.list('ciel/projects/ciel');
    assert.ok(keys.includes('overview'));
    assert.ok(keys.includes('requirements'));
  });

  it('deletes a value', async () => {
    await backend.put('ciel/projects/ciel', 'temp', Buffer.from('temp'));
    await backend.delete('ciel/projects/ciel', 'temp');
    const value = await backend.get('ciel/projects/ciel', 'temp');
    assert.strictEqual(value, null);
  });

  it('searches by query', async () => {
    await backend.put('ciel/kg/concepts', 'obsidian', Buffer.from('Obsidian is a local-first markdown knowledge base'));
    await backend.put('ciel/kg/concepts', 'mcp', Buffer.from('MCP connects agents to tools'));
    const results = await backend.search('ciel/kg/concepts', 'obsidian', 10);
    assert.ok(results.length >= 1);
    assert.ok(results.some(r => r.path.includes('obsidian')));
  });

  it('returns stats for a partition', async () => {
    const stats = await backend.stats('ciel/projects/ciel');
    assert.strictEqual(stats.partition, 'ciel/projects/ciel');
    assert.strictEqual(stats.backend, 'obsidian');
    assert.ok(typeof stats.count === 'number');
  });

  it('self-test reports local-rest-api status ok', async () => {
    const checks = await backend.selfTest();
    const statusCheck = checks.find(c => c.name === 'local-rest-api/status');
    assert.ok(statusCheck);
    assert.strictEqual(statusCheck.ok, true);
  });
});
