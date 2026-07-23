#!/usr/bin/env node
import { ObsidianMemoryBackend } from './adapter.mjs';

const args = process.argv.slice(2);
const cmd = args[0];

function usage() {
  console.log(`
Usage:
  ciel-obsidian-memory --self-test
  ciel-obsidian-memory put <partition> <key> <value>
  ciel-obsidian-memory get <partition> <key>
  ciel-obsidian-memory delete <partition> <key>
  ciel-obsidian-memory list <partition> [prefix]
  ciel-obsidian-memory search <partition> <query> [topK]
  ciel-obsidian-memory stats <partition>

Environment:
  OBSIDIAN_API_URL          default http://127.0.0.1:27123
  OBSIDIAN_API_KEY          required
  OBSIDIAN_VAULT_PATH       required for hybrid search auto-start
  OBSIDIAN_HYBRID_SEARCH_URL default http://127.0.0.1:3939
`);
}

async function main() {
  const backend = new ObsidianMemoryBackend();

  if (!cmd || cmd === '--help' || cmd === '-h') {
    usage();
    process.exit(0);
  }

  if (cmd === '--self-test') {
    const results = await backend.selfTest();
    console.log(JSON.stringify(results, null, 2));
    const allOk = results.every(r => r.ok);
    process.exit(allOk ? 0 : 1);
  }

  if (cmd === 'put') {
    const [partition, key, value] = args.slice(1);
    if (!partition || !key || value === undefined) { usage(); process.exit(1); }
    await backend.put(partition, key, Buffer.from(value));
    console.log('OK');
    return;
  }

  if (cmd === 'get') {
    const [partition, key] = args.slice(1);
    if (!partition || !key) { usage(); process.exit(1); }
    const val = await backend.get(partition, key);
    if (val === null) { console.log('(not found)'); return; }
    console.log(val.toString('utf8'));
    return;
  }

  if (cmd === 'delete') {
    const [partition, key] = args.slice(1);
    if (!partition || !key) { usage(); process.exit(1); }
    await backend.delete(partition, key);
    console.log('OK');
    return;
  }

  if (cmd === 'list') {
    const [partition, prefix] = args.slice(1);
    if (!partition) { usage(); process.exit(1); }
    const keys = await backend.list(partition, prefix || '');
    console.log(JSON.stringify(keys, null, 2));
    return;
  }

  if (cmd === 'search') {
    const [partition, query, topK] = args.slice(1);
    if (!partition || !query) { usage(); process.exit(1); }
    const results = await backend.search(partition, query, Number(topK) || 10);
    console.log(JSON.stringify(results, null, 2));
    return;
  }

  if (cmd === 'stats') {
    const [partition] = args.slice(1);
    if (!partition) { usage(); process.exit(1); }
    const stats = await backend.stats(partition);
    console.log(JSON.stringify(stats, null, 2));
    return;
  }

  console.error(`Unknown command: ${cmd}`);
  usage();
  process.exit(1);
}

main().catch(err => {
  console.error(err.message);
  process.exit(1);
});
