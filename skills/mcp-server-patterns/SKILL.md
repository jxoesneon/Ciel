---
name: mcp-server-patterns
version: 1.0.0
format: skill/1.0
description: Ciel's integration layer for Model Context Protocol (MCP) development.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(build|create|add).*mcp.*(server|tool|resource)"
    confidence: 1.0
  - pattern: "mcp.*integration"
    confidence: 0.95
  - pattern: "@modelcontextprotocol/sdk"
    confidence: 1.0
---

# CIEL ADAPTATION: MCP Server Patterns

This skill provides CIEL's expertise for building, maintaining, and integrating Model Context Protocol (MCP) servers.

## Integration Context

Adapted from `~/.agents/skills/mcp-server-patterns/`. This skill enables CIEL to extend its own capability surface by developing new MCP servers for specialized domains.

## Core Concepts

- **Tools**: Actionable commands (e.g., `git_commit`, `run_query`).
- **Resources**: Read-only data sources (e.g., `file://`, `api://`).
- **Prompts**: Reusable templates for specialized interactions.
- **Transport**: stdio (local) or Streamable HTTP (remote/cloud).

## Orchestration Logic

### 1. Capability Surface Selection
When a new capability is needed, the **orchestration** skill must decide:
- **Skill**: For logic-heavy, multi-session agentic workflows.
- **MCP Tool**: For stateless, reusable atomic actions.
- **CLI/Script**: For simple automation.

### 2. Development Pipeline
Use the **TDD** skill to define tool schemas and expected outputs before implementing the MCP handler.

### 3. Safety & Validation
- Use **Zod** for strict input validation.
- All high-risk tools (destructive actions) must be gated by a **Council of Five** review before registration.
- Ensure proper error handling to prevent raw stack traces from leaking to the model.

### 4. Integration
Newly built MCP servers should be registered in the **MemPalace** Knowledge Graph as part of CIEL's active toolset.

---
*Original Documentation preserved at source.*
