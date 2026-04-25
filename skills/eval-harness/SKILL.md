---
name: eval-harness
version: 1.0.0
format: skill/1.0
description: Formal evaluation framework for CIEL sessions implementing Eval-Driven Development (EDD) principles for autonomous agents.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(evaluate|benchmark|test).*(agent|model|workflow)"

    confidence: 0.9

  - pattern: "setup (eval|edd|eval-driven)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Eval-Harness (Verification Architecture)

This skill formalizes Eval-Driven Development (EDD) within CIEL. It acts as the ultimate quality gate for agent-generated code, prompt engineering, and complex workflows. It treats AI evaluations as "unit tests for agent development."

## Integration Context

Adapted from `~/.agents/skills/eval-harness/`. This skill is reserved for **complex, multi-agent orchestration, autonomous loops, or benchmarking**. For standard, everyday coding tasks, CIEL's default Test-Driven Development (TDD) rule applies. This harness provides the heavy-duty framework for the **Verification-Loop** when formal reliability metrics are required.

## The EDD Lifecycle

### 1. Define (Before Coding)

Before dispatching a sub-agent for a complex task, the Orchestrator MUST define the pass/fail criteria.

- **Capability Evals**: What new thing must the system do?
- **Regression Evals**: What existing things must the system NOT break?

### 2. Grader Types

Evals must be graded using one of three methods:

- **Code-Based Grader**: Deterministic shell commands (e.g., `npm test`, `pytest`, `grep`). *Preferred.*
- **Model-Based Grader**: Using an LLM-as-a-Judge with a strict rubric to evaluate open-ended outputs.
- **Human Grader**: Pausing the workflow for manual review (used for high-risk or subjective changes).

### 3. Metrics

Track agent reliability using formal metrics:

- **pass@1**: Success on the first attempt (Measures prompt clarity).
- **pass@3**: Success within 3 attempts (Measures self-correction capability).
- **pass^3**: Three consecutive successes (Measures absolute stability for critical paths).

## Orchestration Workflow

When triggered to formally evaluate a complex feature or agent workflow:

1. Create an eval definition file (e.g., `.ciel/evals/<feature>.md`).
2. Run the implementation (`subagent-driven-development`).
3. Execute the defined graders against the output.
4. **Conditional**: Only generate a formal report detailing the `pass@k` metrics if explicitly benchmarking or running in a continuous autonomous loop (to prevent context bloat).

## Anti-Patterns

- **Overfitting**: Writing prompts specifically to pass a narrow eval rather than solving the general problem.
- **Flaky Graders**: Using non-deterministic LLM graders for things that could be verified with a simple bash script.
- **Ignoring Drift**: Chasing high pass rates while ignoring exponential token costs or execution latency.

xponential token costs or execution latency.
