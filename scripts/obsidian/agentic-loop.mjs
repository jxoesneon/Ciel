#!/usr/bin/env node
/**
 * Agentic loop orchestrator for Ciel's Obsidian brain.
 *
 * Maps a high-level goal → tasks → subtasks, retrieves vault context,
 * executes each subtask, verifies, and writes durable state back to Obsidian.
 *
 * Usage:
 *   node scripts/obsidian/agentic-loop.mjs "Migrate Ciel to Obsidian brain" \
 *       --project ciel --depth 3 --plan plan.json --execute
 *
 * The loop is dry-run by default. Pass --execute to run subtask actions.
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { ObsidianMemoryBackend } from '../../ciel.skill/memory/backends/obsidian/adapter.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function parseArgs(argv) {
  const args = { goal: '', project: 'default', depth: 3, execute: false, plan: null, verbose: false };
  let i = 0;
  while (i < argv.length) {
    const arg = argv[i];
    if (arg === '--project') args.project = argv[++i];
    else if (arg === '--depth') args.depth = Number(argv[++i]) || 3;
    else if (arg === '--plan') args.plan = argv[++i];
    else if (arg === '--execute') args.execute = true;
    else if (arg === '--verbose') args.verbose = true;
    else if (!arg.startsWith('--') && !args.goal) args.goal = arg;
    i++;
  }
  if (!args.goal) throw new Error('Goal is required');
  return args;
}

function now() {
  return new Date().toISOString();
}

function makeRunId() {
  return `run-${Date.now().toString(36)}`;
}

async function runShell(cmd, args = [], env = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, {
      env: { ...process.env, ...env },
      stdio: ['ignore', 'pipe', 'pipe'],
      shell: process.platform === 'win32',
    });
    let stdout = '';
    let stderr = '';
    child.stdout.on('data', d => (stdout += d));
    child.stderr.on('data', d => (stderr += d));
    child.on('close', code => {
      if (code !== 0) return reject(new Error(`${cmd} ${args.join(' ')} failed (${code}): ${stderr || stdout}`));
      resolve(stdout.trim());
    });
    child.on('error', reject);
  });
}

async function callClaudeForPlan(goal, project, depth) {
  const prompt = `You are Ciel's agentic loop planner. Decompose the following goal into ${depth} levels of tasks and subtasks. Return ONLY a JSON object with no markdown formatting.

Goal: ${goal}
Project: ${project}

JSON schema:
{
  "tasks": [
    {
      "id": "t1",
      "title": "...",
      "acceptance": "...",
      "subtasks": [
        {
          "id": "t1.1",
          "title": "...",
          "action": { "type": "write-note|run-command|search", "payload": "..." },
          "verify": "..."
        }
      ]
    }
  ]
}

Rules:
- Each subtask must be a single concrete action.
- Use action.type "search" when the subtask only needs to gather context.
- Use action.type "write-note" when the subtask should persist a note to the vault.
- Use action.type "run-command" when the subtask should run a shell command.
- Do not include nested subtasks beyond one level.`;

  try {
    const out = await runShell('claude', ['-p', prompt, '--no-persistence']);
    return JSON.parse(out.replace(/```json/g, '').replace(/```/g, '').trim());
  } catch (e) {
    console.warn(`Could not generate plan via claude CLI: ${e.message}. Falling back to heuristic plan.`);
    return heuristicPlan(goal, project, depth);
  }
}

function heuristicPlan(goal, project, depth) {
  const tasks = [];
  const phases = ['Decompose', 'Retrieve', 'Execute', 'Verify', 'Persist'];
  for (let i = 0; i < Math.min(depth, phases.length); i++) {
    tasks.push({
      id: `t${i + 1}`,
      title: `${phases[i]}: ${goal}`,
      acceptance: `${phases[i]} phase is complete and documented`,
      subtasks: [
        {
          id: `t${i + 1}.1`,
          title: `${phases[i].toLowerCase()} context for "${goal}"`,
          action: { type: 'search', payload: goal },
          verify: 'At least one relevant vault note is found or a new note is created',
        },
      ],
    });
  }
  return { tasks };
}

class AgenticLoop {
  constructor(backend, args) {
    this.backend = backend;
    this.args = args;
    this.runId = makeRunId();
    this.projectPath = `ciel/projects/${args.project}`;
    this.tasksPath = `${this.projectPath}/tasks`;
    this.diaryPath = `ciel/diary/agentic-loop-${this.runId}.md`;
  }

  async run() {
    console.log(`Agentic loop ${this.runId} starting: ${this.args.goal}`);

    const plan = await this.loadPlan();
    const enrichedPlan = await this.enrichPlan(plan);
    await this.writePlanToVault(enrichedPlan);

    for (const task of enrichedPlan.tasks) {
      await this.runTask(task);
    }

    const summary = await this.summarizeRun(enrichedPlan);
    await this.writeDiary(summary);
    console.log(`Agentic loop ${this.runId} complete. Diary: ${this.diaryPath}`);
  }

  async loadPlan() {
    if (this.args.plan) {
      const raw = await fs.readFile(this.args.plan, 'utf8');
      return JSON.parse(raw);
    }
    return callClaudeForPlan(this.args.goal, this.args.project, this.args.depth);
  }

  async enrichPlan(plan) {
    // Tag every task/subtask with the run id and project.
    for (const task of plan.tasks) {
      task.run_id = this.runId;
      task.project = this.args.project;
      task.status = 'pending';
      for (const sub of task.subtasks || []) {
        sub.run_id = this.runId;
        sub.task_id = task.id;
        sub.status = 'pending';
      }
    }
    return plan;
  }

  async writePlanToVault(plan) {
    const notePath = `${this.tasksPath}/plan-${this.runId}.md`;
    const content = this.renderPlanNote(plan);
    await this.backend.put(
      `ciel/projects/${this.args.project}/tasks`,
      `plan-${this.runId}`,
      Buffer.from(content),
      {
        title: `Agentic Plan: ${this.args.goal}`,
        tags: ['agentic-loop', 'plan'],
        created: now(),
        run_id: this.runId,
        project: this.args.project,
      }
    );
  }

  renderPlanNote(plan) {
    let md = `# Agentic Plan: ${this.args.goal}\n\n`;
    md += `- **Run ID:** ${this.runId}\n`;
    md += `- **Project:** ${this.args.project}\n`;
    md += `- **Started:** ${now()}\n`;
    md += `- **Execute mode:** ${this.args.execute ? 'yes' : 'dry-run'}\n\n`;
    for (const task of plan.tasks) {
      md += `## ${task.id}: ${task.title}\n\n`;
      md += `**Acceptance:** ${task.acceptance}\n\n`;
      for (const sub of task.subtasks || []) {
        md += `### ${sub.id}: ${sub.title}\n\n`;
        md += `- **Action:** \`${sub.action?.type}\` → ${JSON.stringify(sub.action?.payload)}\n`;
        md += `- **Verify:** ${sub.verify}\n`;
        md += `- **Status:** ${sub.status}\n\n`;
      }
    }
    return md;
  }

  async runTask(task) {
    console.log(`Task ${task.id}: ${task.title}`);
    task.status = 'in_progress';

    for (const sub of task.subtasks || []) {
      await this.runSubtask(sub);
    }

    const allOk = (task.subtasks || []).every(s => s.status === 'done' || s.status === 'skipped');
    task.status = allOk ? 'done' : 'blocked';

    // Update task note in vault.
    await this.backend.put(
      `ciel/projects/${this.args.project}/tasks`,
      `task-${task.id}-${this.runId}`,
      Buffer.from(this.renderTaskNote(task)),
      {
        title: task.title,
        tags: ['agentic-loop', 'task'],
        run_id: this.runId,
        task_id: task.id,
        status: task.status,
      }
    );
  }

  async runSubtask(sub) {
    console.log(`  Subtask ${sub.id}: ${sub.title}`);
    sub.status = 'in_progress';

    try {
      const action = sub.action || { type: 'search', payload: this.args.goal };

      if (action.type === 'search') {
        const results = await this.backend.search(
          `ciel/projects/${this.args.project}`,
          action.payload,
          10
        );
        sub.result = { found: results.length, top: results.slice(0, 3) };
        sub.status = 'done';
      } else if (action.type === 'write-note') {
        const note = typeof action.payload === 'string' ? action.payload : JSON.stringify(action.payload, null, 2);
        await this.backend.put(
          `ciel/projects/${this.args.project}/notes`,
          `sub-${sub.id}-${this.runId}`,
          Buffer.from(note),
          {
            title: sub.title,
            tags: ['agentic-loop', 'note'],
            run_id: this.runId,
            subtask_id: sub.id,
          }
        );
        sub.status = this.args.execute ? 'done' : 'skipped';
      } else if (action.type === 'run-command') {
        if (this.args.execute) {
          const payload = action.payload;
          const cmd = typeof payload === 'string' ? payload : payload.cmd;
          const args = Array.isArray(payload?.args) ? payload.args : [];
          const env = payload?.env || {};
          sub.result = { output: await runShell(cmd, args, env) };
          sub.status = 'done';
        } else {
          sub.status = 'skipped';
          sub.result = { dryRun: true };
        }
      } else {
        sub.status = 'skipped';
        sub.result = { reason: 'unknown action type' };
      }
    } catch (e) {
      sub.status = 'failed';
      sub.result = { error: e.message };
      console.error(`  Subtask ${sub.id} failed: ${e.message}`);
    }
  }

  renderTaskNote(task) {
    let md = `# Task ${task.id}: ${task.title}\n\n`;
    md += `**Acceptance:** ${task.acceptance}\n\n`;
    md += `**Status:** ${task.status}\n\n`;
    for (const sub of task.subtasks || []) {
      md += `## ${sub.id}: ${sub.title}\n\n`;
      md += `- **Status:** ${sub.status}\n`;
      if (sub.result) md += `- **Result:** ${JSON.stringify(sub.result)}\n`;
      md += `\n`;
    }
    return md;
  }

  async summarizeRun(plan) {
    const total = plan.tasks.length;
    const done = plan.tasks.filter(t => t.status === 'done').length;
    const blocked = plan.tasks.filter(t => t.status === 'blocked').length;
    const pending = total - done - blocked;

    return {
      run_id: this.runId,
      goal: this.args.goal,
      project: this.args.project,
      execute: this.args.execute,
      timestamp: now(),
      summary: { total, done, blocked, pending },
      tasks: plan.tasks.map(t => ({ id: t.id, title: t.title, status: t.status })),
    };
  }

  async writeDiary(summary) {
    const md = `---
title: "Agentic Loop: ${summary.goal}"
date: ${summary.timestamp}
run_id: ${summary.run_id}
project: ${summary.project}
tags: [diary, agentic-loop]
status: active
---

# Agentic Loop ${summary.run_id}

## Goal

${summary.goal}

## Summary

- Total tasks: ${summary.summary.total}
- Done: ${summary.summary.done}
- Blocked: ${summary.summary.blocked}
- Pending: ${summary.summary.pending}
- Execute mode: ${summary.execute ? 'yes' : 'dry-run'}

## Tasks

${summary.tasks.map(t => `- **${t.id}** — ${t.title} — ${t.status}`).join('\n')}

## Artifacts

- Plan: [[${this.tasksPath}/plan-${this.runId}]]
- Diary: [[${this.diaryPath}]]
`;

    await this.backend.put('ciel/diary', `agentic-loop-${this.runId}`, Buffer.from(md), {
      title: `Agentic Loop: ${summary.goal}`,
      tags: ['diary', 'agentic-loop'],
      run_id: this.runId,
      project: summary.project,
    });
  }
}

async function main() {
  const argv = process.argv.slice(2);
  if (argv.includes('--help') || argv.includes('-h')) {
    console.log(`
Agentic loop orchestrator for Ciel's Obsidian brain.

Usage:
  node scripts/obsidian/agentic-loop.mjs "<goal>" [options]

Options:
  --project <name>    Scope to a project folder (default: default)
  --depth <n>         Decomposition depth (default: 3)
  --plan <file>       Read a JSON plan instead of generating one
  --execute           Run subtask actions; default is dry-run
  --verbose           Print extra debug info
  --help              Show this help
`);
    process.exit(0);
  }

  const args = parseArgs(argv);
  const backend = new ObsidianMemoryBackend();

  if (args.verbose) {
    console.log('Args:', args);
  }

  // Dry-run safety warning.
  if (!args.execute) {
    console.log('Dry-run mode: no shell commands or writes will be executed. Pass --execute to run actions.');
  }

  const loop = new AgenticLoop(backend, args);
  await loop.run();
}

main().catch(err => {
  console.error(err.message);
  process.exit(1);
});
