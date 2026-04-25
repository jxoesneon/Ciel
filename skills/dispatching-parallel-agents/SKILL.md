---
name: dispatching-parallel-agents
version: 1.0.0
format: skill/1.0
description: Core orchestration pattern for executing multiple independent tasks concurrently using isolated sub-agents.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(dispatch|run|execute).*(parallel|concurrent).*(agents|tasks)"

    confidence: 0.9

  - pattern: "(multiple|several) (independent|unrelated) (failures|bugs|tasks)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Dispatching Parallel Agents (Concurrency Mandate)

This skill formalizes CIEL's approach to concurrency. When the Orchestrator identifies multiple *independent* tasks (e.g., unrelated bug fixes, distinct test file failures), it MUST dispatch specialized sub-agents in parallel rather than executing them sequentially.

## Integration Context

Adapted from `~/.agents/skills/dispatching-parallel-agents/`. This is an advanced execution strategy that builds upon `subagent-driven-development`. While the base skill focuses on isolating context per task, this skill focuses on executing those isolated contexts concurrently to minimize overall turn latency.

## The Concurrency Decision Matrix

Before dispatching in parallel, the Orchestrator MUST evaluate the tasks against these conditions:

**🟢 Proceed with Parallel Dispatch IF:**

- Tasks operate on distinct, non-overlapping files or subsystems.
- Fixing/implementing Task A has zero theoretical impact on Task B.
- The root causes or requirements are entirely independent.

**🔴 Fallback to Sequential Dispatch IF:**

- Tasks require modifying the same files (Shared State).
- Tasks are sequentially dependent (Task B relies on the output of Task A).
- The Orchestrator does not fully understand the system state and is performing exploratory debugging.

## Orchestration Workflow

1. **Identify Domains**: Group failures or requirements into strictly independent domains. Check the Knowledge Graph (`mempalace_kg_query`) for historical implicit dependencies between these domains.
2. **Contextualize**: Craft a highly specific, self-contained prompt for *each* domain. Include the exact error messages, file paths, and success criteria.
3. **Dispatch**: Invoke the sub-agents (e.g., via the `generalist` tool without `wait_for_previous` set to true) concurrently.
4. **Integration Review**:
   - As agents return, read their execution summaries.
   - Run the `verification-loop` (full test suite) to ensure the parallel fixes did not introduce emergent conflicts.
5. **Evolutionary Feedback (Failure Memory)**: If parallel agents produce a silent semantic collision or integration failure despite operating on "non-overlapping" files, the Orchestrator MUST log this implicit dependency to the Knowledge Graph (`mempalace_kg_add` e.g., 'Module A' 'implicitly_depends_on' 'Module B') to prevent future parallel dispatch of these specific domains.

## Anti-Patterns

- **Shared State Collision**: Dispatching parallel agents to edit the same file, resulting in inevitable merge conflicts or overwritten code.
- **Vague Dispatch**: Telling an agent "Fix the bugs" instead of "Fix the timing issue in `auth.test.ts`."
- **Context Bleed**: Passing the entire session history to parallel agents instead of the domain-specific context.
