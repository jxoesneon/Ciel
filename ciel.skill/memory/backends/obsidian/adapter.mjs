import http from 'node:http';
import https from 'node:https';
import { spawn } from 'node:child_process';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import YAML from 'js-yaml';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/**
 * CielMemoryBackend implementation backed by Obsidian.
 *
 * Stack:
 *   - obsidian-local-rest-api for CRUD, periodic notes, and commands.
 *   - obsidian-hybrid-search for semantic + full-text + hybrid retrieval.
 *   - obra/knowledge-graph for graph traversal and community analysis.
 *
 * All values are persisted as markdown files in the Obsidian vault so that
 * Obsidian itself remains the source of truth and a human can read and edit
 * every memory directly.
 */
export class ObsidianMemoryBackend {
  constructor(config = {}) {
    this.apiUrl = config.obsidianApiUrl || process.env.OBSIDIAN_API_URL || 'http://127.0.0.1:27123';
    this.apiKey = config.obsidianApiKey || process.env.OBSIDIAN_API_KEY || '';
    this.vaultPath = config.vaultPath || process.env.OBSIDIAN_VAULT_PATH || '';
    this.hybridSearchUrl = config.hybridSearchUrl || process.env.OBSIDIAN_HYBRID_SEARCH_URL || 'http://127.0.0.1:3939';
    this.kgRepoPath = config.kgRepoPath || process.env.KG_REPO_PATH || path.join(os.homedir(), '.ciel', 'tools', 'knowledge-graph');
    this.kgDataDir = config.kgDataDir || process.env.KG_DATA_DIR || path.join(os.homedir(), '.local', 'share', 'knowledge-graph');
    this.kgVaultPath = config.kgVaultPath || process.env.KG_VAULT_PATH || this.vaultPath;
    this.timeoutMs = config.timeoutMs || 10000;
  }

  /* ------------------------------------------------------------------ */
  /* CielMemoryBackend API                                                */
  /* ------------------------------------------------------------------ */

  async put(partition, key, value, metadata = {}) {
    const filePath = this.#partitionKeyToPath(partition, key);
    const { body, frontmatter } = this.#encodeValue(value, metadata);
    const content = this.#renderMarkdown(frontmatter, body);
    await this.#vaultWrite(filePath, content, 'PUT');
  }

  async get(partition, key) {
    const filePath = this.#partitionKeyToPath(partition, key);
    const content = await this.#vaultRead(filePath);
    if (content === null) return null;
    return this.#decodeValue(content).value;
  }

  async query(partition, filter) {
    // If filter is a simple prefix/string, use the vault file listing.
    if (typeof filter === 'string' || (filter && typeof filter.prefix === 'string')) {
      const prefix = typeof filter === 'string' ? filter : filter.prefix;
      const files = await this.list(partition, prefix);
      const results = [];
      for (const key of files) {
        const value = await this.get(partition, key);
        if (value !== null) results.push({ key, value });
      }
      return results;
    }

    // Structured frontmatter query via obsidian-hybrid-search advanced query syntax.
    const query = this.#buildHybridQuery(partition, filter);
    return this.search(partition, query, filter?.topK || 50);
  }

  async search(partition, query, topK = 10) {
    await this.#ensureHybridSearch();
    const scope = this.#partitionToScope(partition);
    const q = scope ? `${query} folder:${scope}` : query;
    const results = await this.#hybridSearchJson({ query: q, limit: topK });
    return results.map(r => ({
      key: this.#pathToKey(r.path),
      value: null, // lazy; caller can fetch if needed
      score: r.score,
      snippet: r.snippet,
      path: r.path,
    }));
  }

  async delete(partition, key) {
    const filePath = this.#partitionKeyToPath(partition, key);
    await this.#vaultDelete(filePath);
  }

  async list(partition, prefix = '') {
    const dir = this.#partitionToScope(partition);
    const entries = await this.#vaultList(dir);
    return entries
      .filter(e => e.endsWith('.md'))
      .map(e => e.slice(0, -3))
      .filter(k => !prefix || k.startsWith(prefix));
  }

  async compact(partition) {
    // Re-index hybrid search and knowledge graph.
    await this.#reindexHybridSearch();
    if (this.kgVaultPath) {
      await this.#reindexKnowledgeGraph();
    }
  }

  async snapshot(partition, outPath) {
    const sourceDir = path.join(this.vaultPath, this.#partitionToScope(partition));
    await fs.mkdir(outPath, { recursive: true });
    await this.#copyDir(sourceDir, outPath);
  }

  async restore(partition, inPath) {
    const targetDir = path.join(this.vaultPath, this.#partitionToScope(partition));
    await fs.mkdir(targetDir, { recursive: true });
    await this.#copyDir(inPath, targetDir);
    await this.compact(partition);
  }

  async stats(partition) {
    const files = await this.list(partition);
    return {
      partition,
      count: files.length,
      backend: 'obsidian',
      api_url: this.apiUrl,
      vault_path: this.vaultPath,
    };
  }

  /* ------------------------------------------------------------------ */
  /* Knowledge-graph helpers (not part of the strict backend interface)   */
  /* ------------------------------------------------------------------ */

  async kgSearch(query, topK = 10) {
    await this.#ensureHybridSearch();
    return this.#hybridSearchJson({ query, limit: topK });
  }

  async kgRelated(key, depth = 1) {
    if (!this.kgVaultPath) return [];
    return this.#kgCommand('neighbors', key, `--depth ${depth}`);
  }

  async kgPath(fromKey, toKey) {
    if (!this.kgVaultPath) return [];
    return this.#kgCommand('paths', fromKey, toKey);
  }

  async kgCommunities() {
    if (!this.kgVaultPath) return [];
    return this.#kgCommand('communities');
  }

  /* ------------------------------------------------------------------ */
  /* Capability self-test                                                 */
  /* ------------------------------------------------------------------ */

  async selfTest() {
    const checks = [];
    const add = (name, ok, detail) => checks.push({ name, ok, detail });

    try {
      const status = await this.#vaultStatus();
      add('local-rest-api/status', status && status.status === 'OK', status);
    } catch (e) {
      add('local-rest-api/status', false, e.message);
    }

    try {
      const testKey = `test-${Date.now()}`;
      await this.put('__selftest', testKey, Buffer.from('hello'));
      const val = await this.get('__selftest', testKey);
      add('local-rest-api/read-write', val !== null && val.toString() === 'hello', val?.toString());
      await this.delete('__selftest', testKey);
    } catch (e) {
      add('local-rest-api/read-write', false, e.message);
    }

    try {
      await this.#ensureHybridSearch();
      add('hybrid-search/reachable', true, this.hybridSearchUrl);
    } catch (e) {
      add('hybrid-search/reachable', false, e.message);
    }

    try {
      if (this.kgVaultPath) {
        await this.#kgCommand('node', '_CLAUDE.md');
      }
      add('knowledge-graph/reachable', true, this.kgVaultPath);
    } catch (e) {
      add('knowledge-graph/reachable', false, e.message);
    }

    return checks;
  }

  /* ------------------------------------------------------------------ */
  /* Value encoding: markdown with YAML frontmatter                       */
  /* ------------------------------------------------------------------ */

  #encodeValue(value, metadata) {
    let body;
    let isBinary = false;

    // Try to store as UTF-8 text if it is human-readable.
    if (Buffer.isBuffer(value)) {
      const asUtf8 = value.toString('utf8');
      const reEncoded = Buffer.from(asUtf8, 'utf8');
      if (reEncoded.equals(value)) {
        body = asUtf8;
      } else {
        body = value.toString('base64');
        isBinary = true;
      }
    } else if (typeof value === 'string') {
      body = value;
    } else {
      body = JSON.stringify(value, null, 2);
    }

    const frontmatter = {
      ...metadata,
      _ciel_backend: 'obsidian',
      _ciel_enc: isBinary ? 'base64' : 'utf8',
      _ciel_updated: new Date().toISOString(),
    };

    return { body, frontmatter };
  }

  #decodeValue(content) {
    const match = content.match(/^---\n([\s\S]*?)\n---\n+([\s\S]*)$/);
    if (!match) return { value: Buffer.from(content, 'utf8'), metadata: {} };

    const yamlText = match[1];
    const body = match[2];

    const metadata = this.#parseYaml(yamlText) || {};
    const enc = metadata._ciel_enc || 'utf8';
    let value;
    if (enc === 'base64') {
      value = Buffer.from(body, 'base64');
    } else {
      value = Buffer.from(body, 'utf8');
    }
    return { value, metadata };
  }

  #renderMarkdown(frontmatter, body) {
    const yaml = this.#stringifyYaml(frontmatter);
    return `---\n${yaml}---\n\n${body}`;
  }

  #parseYaml(text) {
    try {
      return YAML.load(text);
    } catch {
      return this.#inlineParseYaml(text);
    }
  }

  #inlineParseYaml(text) {
    const obj = {};
    for (const line of text.split('\n')) {
      const idx = line.indexOf(':');
      if (idx === -1) continue;
      const k = line.slice(0, idx).trim();
      let v = line.slice(idx + 1).trim();
      if (v.startsWith('[') && v.endsWith(']')) {
        v = v.slice(1, -1).split(',').map(s => s.trim().replace(/^["']|["']$/g, ''));
      } else if (v === 'true' || v === 'false') {
        v = v === 'true';
      } else if (/^\d+$/.test(v)) {
        v = Number(v);
      }
      obj[k] = v;
    }
    return obj;
  }

  #stringifyYaml(obj) {
    try {
      return YAML.dump(obj, { lineWidth: -1 });
    } catch {
      return Object.entries(obj)
        .map(([k, v]) => {
          if (Array.isArray(v)) return `${k}: [${v.map(s => `"${s}"`).join(', ')}]`;
          if (typeof v === 'boolean') return `${k}: ${v}`;
          return `${k}: ${v}`;
        })
        .join('\n') + '\n';
    }
  }

  /* ------------------------------------------------------------------ */
  /* Path mapping                                                         */
  /* ------------------------------------------------------------------ */

  #partitionKeyToPath(partition, key) {
    const scope = this.#partitionToScope(partition);
    const safeKey = String(key).replace(/[^a-zA-Z0-9._\-]/g, '_');
    return scope ? `${scope}/${safeKey}.md` : `${safeKey}.md`;
  }

  #partitionToScope(partition) {
    if (!partition || partition === 'global') return '';
    return String(partition)
      .replace(/\./g, '/')
      .replace(/^ciel\//, 'ciel/');
  }

  #pathToKey(filePath) {
    return filePath.replace(/\.md$/, '').replace(/\//g, '.');
  }

  #buildHybridQuery(partition, filter) {
    const parts = [];
    if (filter && filter.query) parts.push(filter.query);
    for (const [k, v] of Object.entries(filter || {})) {
      if (k === 'query' || k === 'topK' || k === 'prefix') continue;
      if (Array.isArray(v)) {
        parts.push(`${k}:(${v.join(' OR ')})`);
      } else {
        parts.push(`${k}:${v}`);
      }
    }
    return parts.join(' ');
  }

  /* ------------------------------------------------------------------ */
  /* Local REST API helpers                                               */
  /* ------------------------------------------------------------------ */

  #request(urlPath, method = 'GET', body = null, headers = {}) {
    const url = new URL(urlPath, this.apiUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;
    const allHeaders = {
      Authorization: `Bearer ${this.apiKey}`,
      ...headers,
    };
    if (body !== null && typeof body === 'string' && !allHeaders['Content-Type']) {
      allHeaders['Content-Type'] = 'text/markdown; charset=utf-8';
    }

    return new Promise((resolve, reject) => {
      const req = lib.request(url, { method, headers: allHeaders, timeout: this.timeoutMs }, res => {
        let data = '';
        res.setEncoding('utf8');
        res.on('data', chunk => (data += chunk));
        res.on('end', () => {
          if (res.statusCode >= 200 && res.statusCode < 300) {
            try {
              resolve(data ? JSON.parse(data) : null);
            } catch {
              resolve(data);
            }
          } else {
            reject(new Error(`Obsidian REST API ${method} ${urlPath} returned ${res.statusCode}: ${data}`));
          }
        });
      });
      req.on('error', reject);
      req.on('timeout', () => reject(new Error(`Obsidian REST API ${method} ${urlPath} timed out`)));
      if (body !== null) req.write(body);
      req.end();
    });
  }

  async #vaultStatus() {
    return this.#request('/');
  }

  async #vaultRead(filePath) {
    try {
      return await this.#request(`/vault/${encodeURIComponent(filePath)}`);
    } catch (e) {
      if (e.message.includes('404')) return null;
      throw e;
    }
  }

  async #vaultWrite(filePath, content, method = 'PUT') {
    return this.#request(`/vault/${encodeURIComponent(filePath)}`, method, content, {
      'Content-Type': 'text/markdown; charset=utf-8',
    });
  }

  async #vaultDelete(filePath) {
    return this.#request(`/vault/${encodeURIComponent(filePath)}`, 'DELETE');
  }

  async #vaultList(dirPath) {
    const response = await this.#request(`/vault/${encodeURIComponent(dirPath || '')}`);
    if (!response || typeof response !== 'object') return [];
    return Array.isArray(response) ? response : response.files || [];
  }

  /* ------------------------------------------------------------------ */
  /* Hybrid search helpers                                                */
  /* ------------------------------------------------------------------ */

  async #ensureHybridSearch() {
    try {
      await this.#hybridSearchRequest('/health', 'GET');
    } catch {
      // Try to start the server if it isn't running.
      await this.#spawnHybridSearch();
      await this.#waitForHybridSearch();
    }
  }

  #hybridSearchRequest(urlPath, method = 'POST', body = null) {
    const url = new URL(urlPath, this.hybridSearchUrl);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;
    const headers = { 'Content-Type': 'application/json' };
    return new Promise((resolve, reject) => {
      const req = lib.request(url, { method, headers, timeout: this.timeoutMs }, res => {
        let data = '';
        res.setEncoding('utf8');
        res.on('data', chunk => (data += chunk));
        res.on('end', () => {
          if (res.statusCode >= 200 && res.statusCode < 300) {
            try {
              resolve(data ? JSON.parse(data) : null);
            } catch {
              resolve(data);
            }
          } else {
            reject(new Error(`Hybrid search ${method} ${urlPath} returned ${res.statusCode}: ${data}`));
          }
        });
      });
      req.on('error', reject);
      if (body !== null) req.write(JSON.stringify(body));
      req.end();
    });
  }

  async #hybridSearchJson(params) {
    const results = await this.#hybridSearchRequest('/search', 'POST', params);
    return Array.isArray(results) ? results : results?.results || [];
  }

  async #reindexHybridSearch() {
    try {
      await this.#hybridSearchRequest('/reindex', 'POST');
    } catch {
      // Ignore; index may not be running.
    }
  }

  async #spawnHybridSearch() {
    if (!this.vaultPath) throw new Error('OBSIDIAN_VAULT_PATH is required to start hybrid search server');
    const child = spawn('npx', ['-y', '-p', 'obsidian-hybrid-search@latest', 'obsidian-hybrid-search', 'serve', '--foreground'], {
      env: { ...process.env, OBSIDIAN_VAULT_PATH: this.vaultPath },
      detached: true,
      stdio: 'ignore',
      shell: true,
    });
    return new Promise((resolve, reject) => {
      child.on('error', reject);
      child.on('spawn', () => {
        child.unref();
        resolve();
      });
      setTimeout(() => resolve(), 500);
    });
  }

  async #waitForHybridSearch(retries = 20, delay = 500) {
    for (let i = 0; i < retries; i++) {
      try {
        await this.#hybridSearchRequest('/health', 'GET');
        return;
      } catch {
        await new Promise(r => setTimeout(r, delay));
      }
    }
    throw new Error('Hybrid search server did not start in time');
  }

  /* ------------------------------------------------------------------ */
  /* Knowledge-graph CLI helpers                                          */
  /* ------------------------------------------------------------------ */

  async #kgCommand(command, ...args) {
    return new Promise((resolve, reject) => {
      const env = { ...process.env, KG_VAULT_PATH: this.kgVaultPath, KG_DATA_DIR: this.kgDataDir };
      const useCmd = process.platform === 'win32';
      const child = useCmd
        ? spawn('cmd', ['/c', 'npx', '-y', 'tsx', 'src/cli/index.ts', command, ...args], {
            cwd: this.kgRepoPath,
            env,
            stdio: ['ignore', 'pipe', 'pipe'],
          })
        : spawn('npx', ['-y', 'tsx', 'src/cli/index.ts', command, ...args], {
            cwd: this.kgRepoPath,
            env,
            stdio: ['ignore', 'pipe', 'pipe'],
          });
      let stdout = '';
      let stderr = '';
      child.stdout.on('data', d => (stdout += d));
      child.stderr.on('data', d => (stderr += d));
      child.on('close', code => {
        if (code !== 0) return reject(new Error(`knowledge-graph ${command} failed: ${stderr || stdout}`));
        try {
          resolve(JSON.parse(stdout));
        } catch {
          resolve(stdout);
        }
      });
      child.on('error', reject);
    });
  }

  async #reindexKnowledgeGraph() {
    if (!this.kgVaultPath) return;
    try {
      await this.#kgCommand('index', '--force');
    } catch {
      // knowledge-graph may not be installed; ignore.
    }
  }

  /* ------------------------------------------------------------------ */
  /* Filesystem helpers                                                   */
  /* ------------------------------------------------------------------ */

  async #copyDir(src, dest) {
    await fs.mkdir(dest, { recursive: true });
    const entries = await fs.readdir(src, { withFileTypes: true });
    for (const entry of entries) {
      const srcPath = path.join(src, entry.name);
      const destPath = path.join(dest, entry.name);
      if (entry.isDirectory()) {
        await this.#copyDir(srcPath, destPath);
      } else {
        await fs.copyFile(srcPath, destPath);
      }
    }
  }
}

export default ObsidianMemoryBackend;
