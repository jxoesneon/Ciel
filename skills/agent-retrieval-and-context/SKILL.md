---
name: agent-retrieval-and-context
version: 1.0.0
format: skill/1.0
description: CIEL's framework for iterative context retrieval and subagent context optimization.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(retrieve|gather|refine).*(context|codebase|file)"
    confidence: 0.9
  - pattern: "iterative retrieval"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Agent Retrieval (Progressive Context)

This skill solves the "Context Blindness" problem in multi-agent workflows. it formalizes the loop of searching, evaluating, and refining context before execution.

## The Iterative Loop (Max 3 Cycles)
1. **Dispatch (Broad)**: Start with high-level keywords and domain patterns (e.g., `src/auth/**`).
2. **Evaluate (Score)**: Assess candidate files for relevance (0.0 - 1.0). Identify specific gaps (e.g., "Missing JWT util types").
3. **Refine (Narrow)**: Update search query with discovered terminology and exclude irrelevant paths.
4. **Finalize**: Proceed ONLY when 3+ high-relevance (>= 0.7) files are found or gaps are identified as "Unextractable."

## Context Optimization
- **Token Budget**: Target 3000-5000 tokens for the initial context set.
- **Exclusion Mandate**: Prohibit sending test files, lockfiles, or build artifacts unless explicitly required for a fix.
- **Grep Over Read**: Use `grep_search` to identify relevant blocks before performing a full `read_file`.

## Subagent Handover
- **Context Injection**: Pass only the "High Relevance" files and the "Problem Statement" to subagents.
- **Instructions**: Subagents MUST be told: "Your context is limited; perform one refinement cycle if you lack library definitions."

## Anti-Patterns
- **The Context Dump**: Sending 20+ files to a subagent without relevance scoring.
- **Amnesiac Search**: Searching for the same keywords in every cycle instead of evolving them based on findings.
- **Blind Implementation**: Writing code before identifying the project's naming conventions (e.g., calling it `userSvc` when the codebase uses `AccountManager`).
