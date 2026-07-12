import fs from 'node:fs';
import path from 'node:path';

const root = 'C:/Users/josee/Ciel/obsidian-brain';

const summary = {
  total: 0,
  byFolder: {},
  activeProjects: [],
  concepts: [],
  decisions: [],
  recentDiaries: [],
  sampleFrontmatter: {},
};

function walk(dir, cb) {
  for (const e of fs.readdirSync(dir)) {
    const p = path.join(dir, e);
    const s = fs.statSync(p);
    if (s.isDirectory()) {
      if (e === '.obsidian' || e === '__selftest') continue;
      walk(p, cb);
    } else if (p.endsWith('.md')) cb(p);
  }
}

walk(root, (p) => {
  summary.total++;
  const rel = path.relative(root, p).replace(/\\/g, '/');
  const top = rel.split('/')[0];
  summary.byFolder[top] = (summary.byFolder[top] || 0) + 1;
  const text = fs.readFileSync(p, 'utf8');
  const fm = text.match(/^---\n([\s\S]*?)\n---/);
  if (fm) {
    const keys = fm[1].split('\n').filter(l => l.includes(':')).map(l => l.split(':')[0].trim());
    keys.forEach(k => summary.sampleFrontmatter[k] = (summary.sampleFrontmatter[k] || 0) + 1);
  }
  if (/^ciel\/projects\/[^/]+\/overview\.md$/.test(rel)) {
    const m = text.match(/^title:\s*(.+)$/m);
    if (m) summary.activeProjects.push(m[1].trim().replace(/^["']|["']$/g, ''));
  }
  if (/^ciel\/kg\/concepts\/.+\.md$/.test(rel)) {
    const m = text.match(/^title:\s*(.+)$/m);
    if (m) summary.concepts.push(m[1].trim().replace(/^["']|["']$/g, ''));
  }
  if (/^ciel\/kg\/decisions\/.+\.md$/.test(rel)) {
    const m = text.match(/^title:\s*(.+)$/m);
    if (m) summary.decisions.push(m[1].trim().replace(/^["']|["']$/g, ''));
  }
  if (/^ciel\/diary\/.+\.md$/.test(rel)) {
    const m = text.match(/^title:\s*(.+)$/m) || text.match(/^#\s+(.+)$/m);
    if (m) summary.recentDiaries.push(m[1].trim().replace(/^["']|["']$/g, ''));
  }
});

summary.activeProjects.sort();
summary.concepts.sort();
summary.decisions.sort();
summary.recentDiaries = summary.recentDiaries.slice(-20);

console.log(JSON.stringify(summary, null, 2));
