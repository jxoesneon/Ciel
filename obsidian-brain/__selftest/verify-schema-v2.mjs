import fs from 'node:fs';
import path from 'node:path';

const root = 'C:/Users/josee/Ciel/obsidian-brain';
const allowedTypes = new Set([
  'project', 'project-note', 'concept', 'decision', 'diary', 'person', 'org',
  'index', 'dashboard', 'template', 'system', 'goal', 'task', 'raw', 'wiki', 'note'
]);
const allowedStatuses = new Set([
  'active', 'draft', 'review', 'archived', 'backlog', 'complete', 'completed',
  'proposed', 'adopted', 'rejected', 'superseded', 'pending', 'deferred', 'accepted',
  'decided'
]);

let files = 0, missingFm = 0, dupFm = 0, missingType = 0, badType = 0, badStatus = 0, missingTitle = 0, missingCreated = 0, missingTags = 0, brokenLinks = 0, staleOverview = 0, missingHub = 0;

const fileSet = new Set();
const fileMap = new Map();
function walkDir(dir) {
  for (const e of fs.readdirSync(dir)) {
    const p = path.join(dir, e);
    const s = fs.statSync(p);
    const rel = path.relative(root, p).replace(/\\/g, '/');
    if (s.isDirectory()) {
      if (e === '.obsidian' || e === '__selftest') continue;
      fileSet.add(rel.toLowerCase() + '/');
      walkDir(p);
    } else if (p.endsWith('.md')) {
      const lower = rel.toLowerCase();
      fileSet.add(lower);
      fileSet.add(lower.replace(/\.md$/, ''));
      fileMap.set(lower.replace(/\.md$/, ''), rel);
      fileMap.set(lower, rel);
    }
  }
}
walkDir(root);

function exists(target) {
  if (!target) return false;
  const lower = target.toLowerCase();
  return fileSet.has(lower) || fileSet.has(lower + '.md');
}

const linkRe = /\[\[([^\]|#\]]+?)(?:#[^\]]*)?(?:\|([^\]]+))?\]\]/g;
const projectsDir = path.join(root, 'ciel/projects');
const projectFolders = fs.readdirSync(projectsDir).filter(e => fs.statSync(path.join(projectsDir, e)).isDirectory());

function scanFile(p) {
  const rel = path.relative(root, p).replace(/\\/g, '/');
  if (rel.startsWith('.obsidian/') || rel.startsWith('__selftest/')) return;
  files++;
  const text = fs.readFileSync(p, 'utf8');
  const fm = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!fm) { missingFm++; console.log('missing frontmatter:', rel); return; }
  const rest = text.slice(fm[0].length).trimStart();
  if (rest.startsWith('---')) { dupFm++; console.log('duplicate frontmatter:', rel); }
  const map = {};
  for (const line of fm[1].split(/\r?\n/).filter(Boolean)) {
    const idx = line.indexOf(':');
    if (idx === -1) continue;
    map[line.slice(0, idx).trim().toLowerCase()] = line.slice(idx + 1).trim();
  }
  if (!map.type) { missingType++; console.log('missing type:', rel); }
  else if (!allowedTypes.has(map.type)) { badType++; console.log('bad type', map.type, 'in', rel); }
  if (!map.title) { missingTitle++; console.log('missing title:', rel); }
  if (!map.created) { missingCreated++; console.log('missing created:', rel); }
  if (!map.tags) { missingTags++; console.log('missing tags:', rel); }
  if (map.status && !allowedStatuses.has(String(map.status).toLowerCase())) { badStatus++; console.log('bad status', map.status, 'in', rel); }

  // Strip code blocks and inline code before checking links
  const linkText = text
    .replace(/```[\s\S]*?```/g, '')
    .replace(/`[^`\n]+`/g, '');
  let m;
  while ((m = linkRe.exec(linkText)) !== null) {
    let target = m[1].trim();
    if (!target) continue;
    target = target.replace(/\.md$/i, '');
    if (/^(http|#|\^|!)/.test(target)) continue;
    if (target.toLowerCase().includes('overview')) { staleOverview++; console.log('stale overview link:', rel, '->', m[0]); }
    if (!exists(target) && !rel.endsWith('_CLAUDE.md')) {
      brokenLinks++;
      console.log('broken link:', rel, '->', m[0]);
    }
  }
}

function walkFiles(dir) {
  for (const e of fs.readdirSync(dir)) {
    const p = path.join(dir, e);
    const s = fs.statSync(p);
    if (s.isDirectory()) {
      if (e === '.obsidian' || e === '__selftest') continue;
      walkFiles(p);
    } else if (p.endsWith('.md')) scanFile(p);
  }
}
walkFiles(root);

// Check each project folder has a hub
for (const folder of projectFolders) {
  const hubName = folder === '.github' ? 'github' : folder;
  const hubPath = path.join(projectsDir, folder, `${hubName}.md`);
  if (!fs.existsSync(hubPath)) {
    missingHub++;
    console.log('missing hub:', path.relative(root, hubPath));
  }
}

// Check no overview.md remains in projects
const remainingOverviews = [];
for (const folder of projectFolders) {
  const oldPath = path.join(projectsDir, folder, 'overview.md');
  if (fs.existsSync(oldPath)) remainingOverviews.push(path.relative(root, oldPath));
}

const summary = {
  files, missingFm, dupFm, missingType, badType, badStatus, missingTitle, missingCreated, missingTags, brokenLinks, staleOverview, missingHub, remainingOverviews: remainingOverviews.length
};
console.log(JSON.stringify(summary, null, 2));
if (Object.values(summary).every(v => v === 0 || Array.isArray(v) ? v.length === 0 : true)) console.log('Vault OK');
