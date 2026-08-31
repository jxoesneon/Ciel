---
name: dev-browser
version: 1.0.0
format: skill/1.0
description: CIEL's framework for browser automation with persistent page state. Navigates URLs, fills forms, takes screenshots, scrapes data, and tests web apps via sandboxed JavaScript scripts.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
side_effects: ["shell", "network", "fs"]
triggers:
  - pattern: "(go to|navigate to|open).*(url|website|page|http)"
    confidence: 0.9
  - pattern: "(click on|fill.*form|take.*screenshot|scrape|automate.*browser|test.*website|log into)"
    confidence: 0.9
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: ["dev-browser"] }
---

# CIEL ADAPTATION: Dev Browser (Browser Automation Primitive)

This skill provides a CLI-driven browser automation primitive with persistent page state. It is a catalytic primitive — other skills compose on top of it for e2e testing, scraping, and visual verification. Ciel's autonomy ladder gates network access (logged), secrets (user escalation for form-fill credentials), and fs writes outside project (user escalation).

## Installation

```bash
npm install -g dev-browser
dev-browser install
```

## Capabilities

- **Persistent page state**: Navigation context survives across script invocations within a session.
- **Sandboxed JavaScript scripts**: Execute browser control logic via JS scripts, not interactive sessions.
- **Navigation**: `go to [url]`, click elements, wait for selectors.
- **Form interaction**: Fill fields, submit forms, handle auth flows.
- **Screenshot capture**: Visual snapshots for verification and debugging.
- **Data extraction**: Scrape page content, extract structured data from DOM.
- **Web app testing**: Automated interaction flows for QA and e2e validation.

## Usage Patterns

- **Navigate**: Open a URL and wait for load — `dev-browser` maintains page state between calls.
- **Interact**: Click buttons, fill forms, submit — chain actions in a single script for atomicity.
- **Capture**: Take screenshots at key states for visual evidence in verification pipelines.
- **Extract**: Pull structured data from pages — pair with `regex-vs-llm-structured-text` for parsing.
- **Test**: Automate click-through flows for web app validation — pair with `e2e-testing` for assertions.

## Ciel Autonomy Gates

- **Network access**: Logged by default; no escalation needed for public URLs.
- **Form-fill credentials**: User escalation required — never auto-fill passwords or API keys without explicit user approval.
- **Filesystem writes outside project**: User escalation required — screenshots and scraped data must land in project-scoped paths unless user approves otherwise.

## Integration

- **`e2e-testing`**: Dev-browser provides the browser primitive; e2e-testing provides assertion frameworks.
- **`verification-loop`**: Screenshots serve as visual verification artifacts.
- **`data-scraper-agent`**: Dev-browser handles browser control; scraper agent handles extraction strategy.

## Anti-Patterns

- **Credential Auto-Fill**: Automatically filling login forms with stored credentials without user escalation.
- **Stateless Chaining**: Treating each `dev-browser` call as independent when page state is needed across steps.
- **Unscoped Writes**: Saving screenshots or scraped data outside the project directory without approval.
- **No Wait Strategy**: Clicking elements without waiting for selectors to appear — leads to stale-element failures.
