#!/usr/bin/env node
// read_agy_conversation.js — dump an Antigravity CLI (`agy`) conversation as
// readable text without launching the agy process.
//
// Usage:
//   node read_agy_conversation.js <conversation-id-or-prefix> [--full] [--raw]
//
// Storage layout (Windows):
//   ~/.gemini/antigravity-cli/conversations/<id>.db        SQLite conversation store
//   ~/.gemini/antigravity-cli/brain/<id>/.system_generated/logs/transcript.jsonl      step digest (JSONL)
//   ~/.gemini/antigravity-cli/brain/<id>/.system_generated/logs/transcript_full.jsonl full step content (JSONL)
//   ~/.gemini/antigravity-cli/brain/<id>/.system_generated/tasks/task-*.log           background task logs
//   ~/.gemini/antigravity-cli/annotations/<id>.pbtxt       annotation metadata
//   ~/.gemini/antigravity-cli/presence/<id>.lock           presence lock (mtime ~ last activity)

const fs = require('fs');
const path = require('path');
const os = require('os');

const ROOT = path.join(os.homedir(), '.gemini', 'antigravity-cli');
const args = process.argv.slice(2);
const idArg = args.find(a => !a.startsWith('--'));
const FULL = args.includes('--full');
const RAW = args.includes('--raw');
const MAX = 500;

if (!idArg) {
  // List known conversations when no ID given.
  const convDir = path.join(ROOT, 'conversations');
  if (!fs.existsSync(convDir)) { console.error('No conversations dir at ' + convDir); process.exit(1); }
  for (const f of fs.readdirSync(convDir).filter(f => f.endsWith('.db'))) {
    const id = f.replace(/\.db$/, '');
    const st = fs.statSync(path.join(convDir, f));
    console.log(`${id}  mtime=${st.mtime.toISOString()}  size=${st.size}`);
  }
  process.exit(0);
}

function resolveId(prefix) {
  const convDir = path.join(ROOT, 'conversations');
  const matches = fs.readdirSync(convDir).filter(f => f.startsWith(prefix));
  if (matches.length === 0) return null;
  if (matches.length > 1) { console.error('Ambiguous prefix, matches: ' + matches.join(', ')); process.exit(2); }
  return matches[0].replace(/\.db$/, '');
}

const id = resolveId(idArg);
if (!id) { console.error('No conversation matching: ' + idArg); process.exit(1); }

const brainDir = path.join(ROOT, 'brain', id, '.system_generated');
const candidates = [
  path.join(brainDir, 'logs', 'transcript_full.jsonl'),
  path.join(brainDir, 'logs', 'transcript.jsonl'),
];
const transcript = candidates.find(p => fs.existsSync(p));

console.log(`conversation=${id}`);
console.log(`db=${path.join(ROOT, 'conversations', id + '.db')}`);
if (transcript) console.log(`transcript=${transcript}`);
const taskDir = path.join(brainDir, 'tasks');
if (fs.existsSync(taskDir)) console.log(`task_logs=${fs.readdirSync(taskDir).join(', ')}`);
console.log('---');

if (RAW) {
  process.stdout.write(fs.readFileSync(transcript));
  process.exit(0);
}

if (!transcript) {
  console.log('No JSONL transcript found; inspect the SQLite db directly.');
  process.exit(0);
}

for (const line of fs.readFileSync(transcript, 'utf8').split('\n')) {
  if (!line.trim()) continue;
  let s;
  try { s = JSON.parse(line); } catch { console.log('PARSE_ERR ' + line.slice(0, 120)); continue; }
  let out = `#${s.step_index} [${s.type}/${s.source || ''}] ${s.created_at || ''} `;
  if (s.content) {
    let c = String(s.content).replace(/\s+/g, ' ').trim();
    if (!FULL && c.length > MAX) c = c.slice(0, MAX) + '…';
    out += c;
  }
  if (s.tool_calls) {
    out += ' TOOLCALLS: ' + s.tool_calls.map(t => {
      let a = JSON.stringify(t.args);
      if (!FULL && a.length > 220) a = a.slice(0, 220) + '…';
      return `${t.name}(${a})`;
    }).join(' | ');
  }
  console.log(out);
}
