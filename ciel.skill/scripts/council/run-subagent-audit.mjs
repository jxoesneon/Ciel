#!/usr/bin/env node
/**
 * Council of Five subagent audit runner.
 *
 * Dispatches each Council member as a separate `claude -p` subagent,
 * collects their JSON scores, and synthesizes a Stage 3 Chairman verdict.
 *
 * Usage:
 *   node scripts/council/run-subagent-audit.mjs --case "<path/to/case.md>" --out "<path/to/audit.md>"
 *
 * If no --case is provided, it audits the Obsidian brain migration on this branch.
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..', '..');

const members = ['coherence', 'capability', 'safety', 'efficiency', 'evolution'];

const weights = {
  coherence: 0.20,
  capability: 0.20,
  safety: 0.25,
  efficiency: 0.15,
  evolution: 0.20,
};

const passScore = 6;

function parseArgs(argv) {
  const args = { case: null, out: null, model: null };
  let i = 0;
  while (i < argv.length) {
    const arg = argv[i];
    if (arg === '--case') args.case = argv[++i];
    else if (arg === '--out') args.out = argv[++i];
    else if (arg === '--model') args.model = argv[++i];
    i++;
  }
  return args;
}

async function readMemberPersona(member) {
  const filePath = path.join(repoRoot, 'ciel.skill', 'council', 'members', `${member.toUpperCase()}.md`);
  return fs.readFile(filePath, 'utf8');
}

async function readCase(casePath) {
  if (casePath) {
    return fs.readFile(casePath, 'utf8');
  }
  // Default case: audit the Obsidian brain migration.
  return [
    `# Case: Obsidian Brain Migration Audit`,
    ``,
    `Branch: Obsidian`,
    ``,
    `Artifacts under review:`,
    `- obsidian-brain/ (Obsidian vault starter pack)`,
    `- ciel.skill/memory/backends/obsidian/adapter.mjs (CielMemoryBackend implementation)`,
    `- ciel.skill/memory/backends/obsidian/cli.mjs (CLI + self-test)`,
    `- skills/obsidian-memory/SKILL.md (skill wrapper)`,
    `- scripts/obsidian/agentic-loop.mjs (agentic loop orchestrator)`,
    `- tests/obsidian-memory/adapter.test.mjs (verification tests)`,
    `- docs/OBSIDIAN_BRAIN.md (setup guide)`,
    `- ciel.skill/configuration/global/memory.config.md (backend configuration update)`,
    ``,
    `Constraints:`,
    `- Local-first, privacy-preserving, human-auditable memory.`,
    `- Must implement the existing CielMemoryBackend interface.`,
    `- Must not break existing SQLite/filesystem fallback until explicitly switched.`,
    ``,
    `Success criteria:`,
    `- Passes all five Council lenses.`,
    `- Tests pass against a mock Obsidian REST API.`,
    `- Self-test can verify the live stack.`,
    `- Agentic loop can turn goals into persisted tasks and notes.`,
  ].join('\n');
}

function buildPrompt(persona, member, caseText) {
  return [
    `You are the ${member.toUpperCase()} member of Ciel's Council of Five.`,
    `You are operating as an isolated subagent. Do not reveal your internal reasoning beyond the requested output.`,
    ``,
    `=== PERSONA ===`,
    persona,
    ``,
    `=== CASE ===`,
    caseText,
    ``,
    `=== TASK ===`,
    `Evaluate the case strictly through the ${member.toUpperCase()} lens.`,
    `Return ONLY a JSON object in the following shape (no markdown, no prose, no code fences):`,
    ``,
    JSON.stringify({
      member,
      stage: 1,
      score: 8,
      rationale: '2-3 sentences explaining the score from this lens only.',
      flags: ['flag-name'],
      requests: [],
      veto: false,
    }, null, 2),
    ``,
    `Rules:`,
    `- Score is an integer 0..10.`,
    `- Rationale must stay in your lane (only ${member} concerns).`,
    `- Flags must be from the taxonomy in your persona file.`,
    `- Set veto: true only if you are Safety and your score is <= 3.`,
    `- Do not include any text outside the JSON object.`,
  ].join('\n');
}

async function runClaude(prompt, modelArg) {
  return new Promise((resolve, reject) => {
    const args = ['-p', prompt, '--no-persistence'];
    if (modelArg) args.push('--model', modelArg);
    const child = spawn('claude', args, {
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', d => (stdout += d));
    child.stderr.on('data', d => (stderr += d));
    child.on('close', code => {
      if (code !== 0) return reject(new Error(`claude subagent failed (${code}): ${stderr || stdout}`));
      resolve(stdout.trim());
    });
    child.on('error', reject);
  });
}

function extractJson(text) {
  // Try to extract JSON from a fenced block or raw text.
  const blockMatch = text.match(/```json\s*([\s\S]*?)\s*```/);
  if (blockMatch) return blockMatch[1].trim();
  return text.trim();
}

function parseResult(text, member) {
  try {
    const json = extractJson(text);
    return JSON.parse(json);
  } catch (e) {
    return {
      member,
      stage: 1,
      score: 0,
      rationale: `Failed to parse subagent output: ${e.message}. Raw output: ${text.slice(0, 200)}`,
      flags: ['parse_error'],
      requests: [],
      veto: false,
    };
  }
}

async function runMember(member, caseText, modelArg) {
  const persona = await readMemberPersona(member);
  const prompt = buildPrompt(persona, member, caseText);
  console.log(`Running ${member} subagent...`);
  const output = await runClaude(prompt, modelArg);
  return parseResult(output, member);
}

function synthesize(stage1Results) {
  const scores = {};
  for (const r of stage1Results) scores[r.member] = r.score;

  let weighted = 0;
  for (const [member, weight] of Object.entries(weights)) {
    weighted += (scores[member] || 0) * weight;
  }

  const aboveThreshold = Object.values(scores).filter(s => s >= passScore).length;
  const safetyScore = scores.safety || 0;
  const veto = safetyScore <= 3;

  let verdict;
  if (veto) verdict = 'REJECT (Safety veto)';
  else if (aboveThreshold >= 3 && weighted >= 6.5) verdict = 'PASS';
  else if (weighted <= 4.5) verdict = 'REJECT';
  else verdict = 'REVIEW / MITIGATIONS REQUIRED';

  return { weighted, scores, aboveThreshold, safetyScore, veto, verdict };
}

function renderAudit(caseText, stage1Results, synthesis) {
  const now = new Date().toISOString();
  const lines = [
    '---',
    `title: Council Subagent Audit — Obsidian Brain Migration`,
    `tags: [decision, adr, audit, ciel]`,
    `project: ciel`,
    `decision_date: ${now.split('T')[0]}`,
    `status: adopted`,
    `method: subagent`,
    '---',
    '',
    '# Council Subagent Audit — Obsidian Brain Migration',
    '',
    'This audit was produced by running each Council of Five member as a separate `claude -p` subagent.',
    '',
    '## Case',
    '',
    caseText,
    '',
    '## Stage 1 — Subagent Scores',
    '',
    '| Member | Score | Veto | Flags | Rationale |',
    '| --- | --- | --- | --- | --- |',
  ];

  for (const r of stage1Results) {
    const flags = (r.flags || []).join(', ') || 'none';
    const veto = r.veto ? 'YES' : 'no';
    lines.push(`| ${r.member} | ${r.score} | ${veto} | ${flags} | ${r.rationale} |`);
  }

  lines.push('');
  lines.push('## Stage 3 — Chairman Synthesis');
  lines.push('');
  lines.push(`- **Weighted score:** ${synthesis.weighted.toFixed(2)} / 10`);
  lines.push(`- **Members above threshold (${passScore}):** ${synthesis.aboveThreshold} / 5`);
  lines.push(`- **Safety score:** ${synthesis.safetyScore}`);
  lines.push(`- **Veto:** ${synthesis.veto ? 'yes' : 'no'}`);
  lines.push(`- **Verdict:** **${synthesis.verdict}**`);
  lines.push('');
  lines.push('## Required Actions');
  lines.push('');
  const requests = new Set();
  for (const r of stage1Results) {
    for (const req of r.requests || []) requests.add(req);
  }
  if (requests.size === 0) {
    lines.push('- None recorded by subagents.');
  } else {
    for (const req of requests) lines.push(`- ${req}`);
  }
  lines.push('');
  lines.push('## Related');
  lines.push('');
  lines.push('- [[ciel/kg/decisions/obsidian-brain-migration-audit]] — prior monolithic audit');
  lines.push('- [[_CLAUDE.md]]');
  lines.push('');

  return lines.join('\n');
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const caseText = await readCase(args.case);
  const outPath = args.out || path.join(repoRoot, 'obsidian-brain', 'ciel', 'kg', 'decisions', 'obsidian-brain-migration-audit-subagents.md');

  const stage1Results = [];
  for (const member of members) {
    const result = await runMember(member, caseText, args.model);
    stage1Results.push(result);
    console.log(JSON.stringify(result, null, 2));
  }

  const synthesis = synthesize(stage1Results);
  console.log('\nSynthesis:', JSON.stringify(synthesis, null, 2));

  const audit = renderAudit(caseText, stage1Results, synthesis);
  await fs.writeFile(outPath, audit, 'utf8');
  console.log(`\nAudit written to: ${outPath}`);
}

main().catch(err => {
  console.error(err.message);
  process.exit(1);
});
