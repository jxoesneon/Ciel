---
name: jira-integration
version: 1.0.0
format: skill/1.0
description: CIEL's protocol for interacting with Jira issues, sprints, and development information.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:

  - pattern: "(jira|atlassian).*(ticket|issue|sprint|jql)"

    confidence: 0.9

  - pattern: "get (requirements|ac) from (ticket|jira)"

    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: Jira Integration (Issue Tracking)

This skill formalizes how CIEL interacts with Jira for requirements gathering, status updates, and development tracking. It prioritizes the `mcp-atlassian` server for tool-based interaction.

## Core Interaction Modes

1. **MCP (Preferred)**: Uses `jira_get_issue`, `jira_search`, and `jira_transition_issue` for structured, authenticated operations.
2. **REST Fallback**: Uses `curl` with `JIRA_API_TOKEN` for environments where MCP is unavailable.

## Requirements Extraction Protocol

When reading a ticket, the Orchestrator MUST extract:

- **Testable AC**: Specific, binary conditions for success.
- **Functional Scope**: What is being changed vs. what is out of scope.
- **Integration Points**: Affected services, APIs, or database schemas.
- **Edge Cases**: Negative tests and boundary conditions defined in the ticket.

## TICKET UPDATE Lifecycle

- **START**: Transition to `In Progress` and comment with the implementation branch name.
- **VERIFY**: Comment with a summary of test results and coverage after the `verification-loop` passes.
- **FINISH**: Transition to `Done` or `In Review` and link the final PR/Commit.

## Anti-Patterns

- **Secret Leaking**: Hardcoding API tokens in scripts or session notes.
- **Vague Updates**: Commenting "Working on it" without providing a branch or specific task context.
- **Status Drift**: Implementing features that contradict the ticket's Acceptance Criteria without clarifying first.
