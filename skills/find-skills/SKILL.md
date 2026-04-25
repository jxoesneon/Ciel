---
name: find-skills
version: 1.0.0
format: skill/1.0
description: Discovery and installation of agent skills from the open ecosystem.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(find|search|lookup|is there).*skill"
    confidence: 0.9
  - pattern: "how do I.*(react|testing|design|deploy)"
    confidence: 0.8
  - pattern: "can you do.*(specialized|expert)"
    confidence: 0.7
  - pattern: "npx skills.*"
    confidence: 0.9

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Find Skills

This skill facilitates the discovery and installation of modular capabilities from the open agent skills ecosystem.

## Integration Context

Adapted from `~/.agents/skills/find-skills/`. This skill serves as CIEL's gateway to external capability acquisition.

## Core Commands (via npx skills)

- `find [query]` — Interactive search
- `add <package>` — Install a skill (use `-g -y` for CIEL-managed installs)
- `check` — Check for updates
- `update` — Update all skills

## Orchestration Logic

### 1. Gap Detection
When CIEL detects a missing capability (e.g., "I don't have a tool for X"), this skill is triggered to search the ecosystem.

### 2. Quality Filter
Before proposing an install, verify:
- **Installs**: Prefer >1K.
- **Source**: Trusted (vercel-labs, anthropics, etc.).
- **Stars**: Prefer >100.

### 3. Installation
CIEL can execute `npx skills add <owner/repo@skill> -g -y` to acquire the skill. Once installed, CIEL must run its own **Ingestion Protocol** to harmonize the new skill.

---
*Original Documentation preserved at source.*
