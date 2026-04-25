---
name: continuous-agent-loop
version: 1.0.0
format: skill/1.0
description: CIEL's canonical loop patterns for autonomous execution with quality gates and recovery controls.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(start|run|select).*(loop|pattern|stack)"
    confidence: 0.9
  - pattern: "how should I (run|execute) this (autonomous|continuous) task"
    confidence: 0.85
---

# CIEL ADAPTATION: Continuous Agent Loop (Execution Stacks)

This skill formalizes the selection and management of autonomous execution loops in CIEL. It supersedes `autonomous-loops` by introducing a structured "Loop Selection Flow" based on task requirements.

## Loop Selection Flow
1. **Sequential**: Standard task-by-task execution. (Default)
2. **PR/Continuous**: For tasks requiring strict CI/PR integration and branch control.
3. **RFC-DAG**: For complex systems-level changes requiring formal decomposition (`ralphinho-rfc-pipeline`).
4. **Infinite/Exploratory**: For brainstorming or parallel generation tasks without a fixed endpoint.

## The Production Stack
A standard CIEL production loop consists of:
1. **Decomposition**: Breaking the epic into an RFC-driven DAG.
2. **Quality Gates**: Enforcing `plankton-code-quality` at every write boundary.
3. **Evaluation**: Running `eval-harness` to measure behavioral correctness.
4. **Persistence**: Maintaining state across session restarts via `nanoclaw-repl`.

## Failure & Recovery
- **Churn Detection**: If a loop repeats the same diagnosis 3 times without a successful verification, it is marked as "Churning."
- **Freeze & Audit**: Churning loops MUST be frozen. The Orchestrator MUST run `/harness-audit` and reduce scope to the smallest failing unit before resuming.

## Anti-Patterns
- **Unbounded Loops**: Running a loop without a clear stop condition or cost cap.
- **Bypassing Gates**: Disabling quality gates or tests to "speed up" the loop.
- **Silent Failures**: Allowing a loop to continue after a critical tool failure without re-running the `verification-loop`.
