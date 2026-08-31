---
name: zai-cli
version: 1.0.0
format: skill/1.0
description: CIEL's framework for Z.AI CLI integration. Provides vision analysis, real-time web search, page reading, and GitHub code exploration via a single npx-based tool.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(z\\.ai|zai-cli|zai cli).*(vision|search|read|repo|analyze)"
    confidence: 0.9
  - pattern: "(analyze|ocr|describe).*(image|screenshot|video).*(zai|z\\.ai)"
    confidence: 0.85
source: { tier: 2, origin: "zai-cli" }
dependencies: { skills: [], mcp: [], system: ["npx", "node"] }
side_effects: ["network", "external_api"]
---

# CIEL ADAPTATION: ZAI CLI

This skill provides access to Z.AI capabilities via `npx zai-cli` — a self-documenting CLI offering vision analysis (GLM-4.6V), real-time web search, web-to-markdown extraction, GitHub code exploration, and MCP tool discovery. Requires `Z_AI_API_KEY`. Ciel's autonomy ladder escalates secrets to the user — the API key must be provided by the user, never auto-generated or hardcoded.

## Setup

```bash
export Z_AI_API_KEY="your-api-key"
```

Get a key at: https://z.ai/manage-apikey/apikey-list

## Commands

| Command | Purpose |
|---------|---------|
| `vision` | Analyze images, screenshots, videos (8 subcommands) |
| `search` | Real-time web search with domain/recency filtering |
| `read` | Fetch web pages as markdown |
| `repo` | GitHub code search and reading via ZRead |
| `tools` | List available MCP tools |
| `tool` | Show tool schema |
| `call` | Raw MCP tool invocation |
| `code` | TypeScript tool chaining |
| `doctor` | Check setup and connectivity |

## Quick Start

```bash
# Analyze an image
npx zai-cli vision analyze ./screenshot.png "What errors do you see?"

# Search the web
npx zai-cli search "React 19 new features" --count 5

# Read a web page
npx zai-cli read https://docs.example.com/api --with-images-summary --no-gfm

# Explore a GitHub repo
npx zai-cli repo search facebook/react "server components"
npx zai-cli repo tree openai/codex --path codex-rs --depth 2

# Check setup
npx zai-cli doctor
```

## Output Format

- **Default**: data-only (raw output for token efficiency).
- **JSON mode**: `--output-format json` wraps in `{ success, data, timestamp }`.

## Anti-Patterns

- **Hardcoded API key**: Never embed `Z_AI_API_KEY` in source files or configs — use env vars only.
- **Skipping doctor check**: Always run `npx zai-cli doctor` first to verify connectivity before complex operations.
- **Verbose output in pipelines**: Use default data-only format; reserve `--output-format json` for programmatic consumption.
- **Ignoring rate limits**: Z.AI API has rate limits — batch requests and handle 429 responses gracefully.
