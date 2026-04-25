---
name: timeline-report
version: 1.0.0
format: skill/1.0
description: Generates a narrative history of a project's evolution using claude-mem observations.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:

  - pattern: "(generate|write).*(timeline|journey|history).*(report)"

    confidence: 0.9

  - pattern: "what is the story of this project"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Timeline Report (Technical History)

This skill leverages CIEL's persistent memory (via `claude-mem`) to generate a comprehensive "Journey Report" for a project, analyzing pivots, breakthroughs, and technical debt accumulation over time.

## Workflow

1. **Identify Parent**: If in a git worktree, resolve the parent project name to ensure correct data retrieval.
2. **Fetch Observations**: Query the `claude-mem` worker API (port 37777) for the full compressed timeline.
3. **Analyze Trends**:
   - **Architectural Evolution**: When did the data model shift? Why?
   - **Key Breakthroughs**: Identify the "Aha!" moments in investigation sessions.
   - **Debugging Sagas**: Track the hardest problems and the turns invested to solve them.
4. **Token ROI**: Perform a quantitative analysis of how much re-work was saved by context injection.

## Output Structure

- **Project Genesis**: Initial vision and founding technical decisions.
- **The Story Arc**: Narrate the pivots from investigation to resolution.
- **Memory Impact**: Evidence of where recalled context saved time or prevented mistakes.
- **Quantitative ROI**: Monthly breakdown of discovery tokens vs. read tokens.

## Anti-Patterns

- **Flat Summarization**: Listing observations chronologically without identifying the narrative arc or pivots.
- **Ignoring ROI**: Failing to quantify the actual efficiency gains provided by the memory system.
- **Surface Analysis**: Reporting what was done without explaining *why* the architecture evolved.
