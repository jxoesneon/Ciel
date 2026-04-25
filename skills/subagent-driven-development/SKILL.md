---
name: subagent-driven-development
version: 1.0.0
format: skill/1.0
description: Core CIEL orchestration pattern for delegating complex, multi-step implementation plans to ephemeral, specialized sub-agents.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "execute (plan|tasks|implementation)"
    confidence: 0.95
  - pattern: "(delegate|dispatch).*(task|subagent)"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Subagent-Driven Development (Delegation Mandate)

This skill formalizes CIEL's orchestration philosophy regarding context management and task execution. To preserve the Orchestrator's context window, independent implementation tasks MUST be delegated to fresh, ephemeral sub-agents (e.g., the `generalist` tool or specific skill agents).

## Integration Context

Adapted from `~/.agents/skills/subagent-driven-development/`. This skill represents the transition from the **Strategy Layer** (`make-plan`) to the **Execution Layer** (`do`).

## The Core Loop (Per Task)

When executing an approved implementation plan, the Orchestrator executes the following loop for each discrete task:

### 1. Dispatch (The Implementer)
- Extract the specific task, relevant context, and strict acceptance criteria from the plan.
- Dispatch a sub-agent (e.g., `generalist`) with a highly focused prompt.
- **Rule**: Do NOT pass the entire session history. Pass ONLY what the sub-agent needs to succeed.

### 2. Stage 1: Spec Compliance Review (The Verifier)
- Once the Implementer returns, the Orchestrator (or a secondary sub-agent) evaluates the diff strictly against the original task specification.
- **Goal**: Did it build exactly what was asked, without over-engineering (YAGNI) or missing requirements?
- If it fails, dispatch a sub-agent to fix the specific gaps.

### 3. Stage 2: Code Quality Review (The Auditor)
- After Spec Compliance is verified, evaluate the diff against CIEL's `coding-standards`.
- **Goal**: Is it readable? Does it respect immutability? Are there code smells (deep nesting, long functions)?
- If it fails, dispatch a sub-agent to refactor for quality.

## Orchestrator Responsibilities

- **Blocker Resolution**: If a sub-agent reports it is `BLOCKED` or `NEEDS_CONTEXT`, the Orchestrator must analyze the failure, gather the missing context (using `read_file`, `grep_search`, etc.), and re-dispatch with a refined prompt. Do NOT attempt to manually implement the fix in the main session.
- **Sequential Safety**: Ensure tasks that depend on the same files are not dispatched concurrently to avoid race conditions.

## Anti-Patterns

- **Context Pollution**: Performing heavy implementation work directly in the main Orchestrator session.
- **Review Bypassing**: Accepting sub-agent output without running both the Spec Compliance and Code Quality reviews.
- **Blind Retries**: Re-dispatching a failing sub-agent without changing its context, prompt, or model assignment.
