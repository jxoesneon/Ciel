---
name: blueprint
version: 1.0.0
format: skill/1.0
description: CIEL's framework for multi-session construction planning. Turns a one-line objective into a step-by-step plan with adversarial review gates, dependency graphs, parallel step detection, and self-contained per-step context briefs.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:strategy"]
triggers:
  - pattern: "(blueprint|construction plan|multi-session plan|multi-agent plan)"
    confidence: 0.9
  - pattern: "(plan|roadmap).*(multi-PR|multi-session|multi-agent|parallel)"
    confidence: 0.85
source: { tier: 1, origin: "blueprint" }
dependencies: { skills: [], mcp: [], system: ["git", "gh"] }
side_effects: []
---

# CIEL ADAPTATION: Blueprint — Construction Plan Generator

This skill turns a one-line objective into a step-by-step construction plan that any coding agent can execute cold. Unlike `make-plan` (which produces a simple linear task list), Blueprint adds adversarial review gates, dependency graphs with parallel step detection, an anti-pattern catalog, a plan mutation protocol, and self-contained per-step context briefs designed for multi-session and multi-agent use. Pure planning — no execution, zero runtime risk.

## When to Use

- Breaking a large feature into multiple PRs with clear dependency order
- Planning a refactor or migration spanning multiple sessions
- Coordinating parallel workstreams across sub-agents
- Any task where context loss between sessions would cause rework

**Do not use** for tasks completable in a single PR, fewer than 3 tool calls, or when the user says "just do it."

## 5-Phase Pipeline

1. **Research** — Pre-flight checks (git, gh auth, remote, default branch). Reads project structure, existing plans, and memory files for context.
2. **Design** — Breaks objective into one-PR-sized steps (3–12 typical). Assigns dependency edges, parallel/serial ordering, model tier (strongest vs default), and rollback strategy per step.
3. **Draft** — Writes self-contained Markdown plan to `plans/`. Every step includes context brief, task list, verification commands, and exit criteria — a fresh agent can execute any step without reading prior steps.
4. **Review** — Delegates adversarial review to a strongest-model sub-agent against a checklist and anti-pattern catalog. Fixes all critical findings before finalizing.
5. **Register** — Saves plan, updates memory index, presents step count and parallelism summary.

## Key Features

- **Cold-start execution**: Every step has a self-contained context brief — no prior context needed.
- **Adversarial review gate**: Strongest-model sub-agent reviews every plan for completeness, dependency correctness, and anti-patterns.
- **Branch/PR/CI workflow**: Built into every step; degrades gracefully to direct mode when git/gh is absent.
- **Parallel step detection**: Dependency graph identifies steps with no shared files or output dependencies.
- **Plan mutation protocol**: Steps can be split, inserted, skipped, reordered, or abandoned with formal protocols and audit trail.

## Distinction from make-plan

Blueprint is for complex, multi-session, multi-agent work. `make-plan` is for straightforward linear tasks. Blueprint's additions: adversarial review, dependency graphs, parallel detection, anti-pattern catalog, mutation protocol, and per-step cold-start context briefs.

## Anti-Patterns

- **Using Blueprint for simple tasks**: If it fits in one PR or fewer than 3 tool calls, use `make-plan` instead — Blueprint's overhead is wasted.
- **Skipping the review gate**: The adversarial review phase catches dependency errors and anti-patterns — never finalize without it.
- **Non-self-contained steps**: Every step must include enough context for cold-start execution — if a step requires reading prior steps, the context brief is incomplete.
- **Ignoring parallelism**: Failing to detect parallel steps serializes work unnecessarily — always analyze the dependency graph for independent branches.
